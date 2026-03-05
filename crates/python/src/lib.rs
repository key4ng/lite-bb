use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use bb_core::api::client::Client;
use bb_core::auth::Credentials;
use bb_core::config::Config;
use bb_core::models::{
    Branch, CommentContent, CreateComment, CreatePullRequest, Destination, MergeRequest,
    UpdatePullRequest,
};

fn to_py_err(e: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}

fn build_client() -> PyResult<Client> {
    let config = Config::load().map_err(to_py_err)?;
    let credentials = config.credentials().map_err(to_py_err)?;
    Client::new(&credentials).map_err(to_py_err)
}

/// Serialize a Rust value to a Python object via JSON string + json.loads()
fn to_pyobj<T: serde::Serialize>(py: Python<'_>, value: &T) -> PyResult<PyObject> {
    let json_str = serde_json::to_string(value).map_err(to_py_err)?;
    let json_mod = py.import("json")?;
    let result = json_mod.call_method1("loads", (json_str,))?;
    Ok(result.unbind())
}

fn rt() -> PyResult<tokio::runtime::Runtime> {
    tokio::runtime::Runtime::new().map_err(to_py_err)
}

// -- Auth --

#[pyfunction]
#[pyo3(signature = (token=None, username=None, app_password=None))]
fn auth_login(
    token: Option<String>,
    username: Option<String>,
    app_password: Option<String>,
) -> PyResult<()> {
    let mut config = Config::load().unwrap_or_default();

    if let Some(t) = token {
        config.token = Some(t);
        config.username = None;
        config.app_password = None;
    } else if let (Some(u), Some(p)) = (username, app_password) {
        config.token = None;
        config.username = Some(u);
        config.app_password = Some(p);
    } else {
        return Err(PyRuntimeError::new_err(
            "provide either token= or both username= and app_password=",
        ));
    }

    config.save().map_err(to_py_err)
}

#[pyfunction]
fn auth_status() -> PyResult<String> {
    let config = Config::load().map_err(to_py_err)?;
    match config.credentials() {
        Ok(Credentials::Token(_)) => Ok("token".to_string()),
        Ok(Credentials::AppPassword { username, .. }) => Ok(format!("app_password:{username}")),
        Err(e) => Err(to_py_err(e)),
    }
}

// -- PR --

#[pyfunction]
#[pyo3(signature = (workspace, repo, state=None, limit=None))]
fn pr_list(
    py: Python<'_>,
    workspace: &str,
    repo: &str,
    state: Option<&str>,
    limit: Option<u32>,
) -> PyResult<PyObject> {
    let client = build_client()?;
    let result = rt()?
        .block_on(client.list_prs(workspace, repo, state, None, limit))
        .map_err(to_py_err)?;
    to_pyobj(py, &result.values)
}

#[pyfunction]
fn pr_get(py: Python<'_>, workspace: &str, repo: &str, id: u64) -> PyResult<PyObject> {
    let client = build_client()?;
    let pr = rt()?
        .block_on(client.get_pr(workspace, repo, id))
        .map_err(to_py_err)?;
    to_pyobj(py, &pr)
}

#[pyfunction]
#[pyo3(signature = (workspace, repo, title, source_branch, destination_branch=None, description=None))]
fn pr_create(
    py: Python<'_>,
    workspace: &str,
    repo: &str,
    title: &str,
    source_branch: &str,
    destination_branch: Option<&str>,
    description: Option<&str>,
) -> PyResult<PyObject> {
    let client = build_client()?;
    let request = CreatePullRequest {
        title: title.to_string(),
        source: Destination {
            branch: Branch {
                name: source_branch.to_string(),
            },
            repository: None,
        },
        destination: destination_branch.map(|b| Destination {
            branch: Branch {
                name: b.to_string(),
            },
            repository: None,
        }),
        description: description.map(|s| s.to_string()),
        close_source_branch: None,
        reviewers: None,
    };
    let pr = rt()?
        .block_on(client.create_pr(workspace, repo, &request))
        .map_err(to_py_err)?;
    to_pyobj(py, &pr)
}

#[pyfunction]
#[pyo3(signature = (workspace, repo, id, title=None, description=None, destination_branch=None))]
fn pr_edit(
    py: Python<'_>,
    workspace: &str,
    repo: &str,
    id: u64,
    title: Option<&str>,
    description: Option<&str>,
    destination_branch: Option<&str>,
) -> PyResult<PyObject> {
    let client = build_client()?;
    let update = UpdatePullRequest {
        title: title.map(|s| s.to_string()),
        description: description.map(|s| s.to_string()),
        destination: destination_branch.map(|b| Destination {
            branch: Branch {
                name: b.to_string(),
            },
            repository: None,
        }),
    };
    let pr = rt()?
        .block_on(client.update_pr(workspace, repo, id, &update))
        .map_err(to_py_err)?;
    to_pyobj(py, &pr)
}

#[pyfunction]
#[pyo3(signature = (workspace, repo, id, strategy=None, message=None))]
fn pr_merge(
    py: Python<'_>,
    workspace: &str,
    repo: &str,
    id: u64,
    strategy: Option<&str>,
    message: Option<&str>,
) -> PyResult<PyObject> {
    let client = build_client()?;
    let request = MergeRequest {
        merge_strategy: strategy.map(|s| s.to_string()),
        close_source_branch: Some(true),
        message: message.map(|s| s.to_string()),
    };
    let pr = rt()?
        .block_on(client.merge_pr(workspace, repo, id, &request))
        .map_err(to_py_err)?;
    to_pyobj(py, &pr)
}

#[pyfunction]
fn pr_close(py: Python<'_>, workspace: &str, repo: &str, id: u64) -> PyResult<PyObject> {
    let client = build_client()?;
    let pr = rt()?
        .block_on(client.decline_pr(workspace, repo, id))
        .map_err(to_py_err)?;
    to_pyobj(py, &pr)
}

#[pyfunction]
fn pr_approve(workspace: &str, repo: &str, id: u64) -> PyResult<()> {
    let client = build_client()?;
    rt()?
        .block_on(client.approve_pr(workspace, repo, id))
        .map_err(to_py_err)
}

#[pyfunction]
fn pr_diff(workspace: &str, repo: &str, id: u64) -> PyResult<String> {
    let client = build_client()?;
    rt()?
        .block_on(client.get_diff(workspace, repo, id))
        .map_err(to_py_err)
}

#[pyfunction]
fn pr_comments(py: Python<'_>, workspace: &str, repo: &str, id: u64) -> PyResult<PyObject> {
    let client = build_client()?;
    let result = rt()?
        .block_on(client.list_comments(workspace, repo, id))
        .map_err(to_py_err)?;
    to_pyobj(py, &result.values)
}

#[pyfunction]
fn pr_comment(
    py: Python<'_>,
    workspace: &str,
    repo: &str,
    id: u64,
    body: &str,
) -> PyResult<PyObject> {
    let client = build_client()?;
    let comment = rt()?
        .block_on(client.add_comment(
            workspace,
            repo,
            id,
            &CreateComment {
                content: CommentContent {
                    raw: Some(body.to_string()),
                    markup: Some("markdown".to_string()),
                    html: None,
                },
            },
        ))
        .map_err(to_py_err)?;
    to_pyobj(py, &comment)
}

#[pyfunction]
fn pr_checks(py: Python<'_>, workspace: &str, repo: &str, id: u64) -> PyResult<PyObject> {
    let client = build_client()?;
    let result = rt()?
        .block_on(client.get_statuses(workspace, repo, id))
        .map_err(to_py_err)?;
    to_pyobj(py, &result.values)
}

#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[pymodule]
fn bb(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;

    // Auth
    m.add_function(wrap_pyfunction!(auth_login, m)?)?;
    m.add_function(wrap_pyfunction!(auth_status, m)?)?;

    // PR
    m.add_function(wrap_pyfunction!(pr_list, m)?)?;
    m.add_function(wrap_pyfunction!(pr_get, m)?)?;
    m.add_function(wrap_pyfunction!(pr_create, m)?)?;
    m.add_function(wrap_pyfunction!(pr_edit, m)?)?;
    m.add_function(wrap_pyfunction!(pr_merge, m)?)?;
    m.add_function(wrap_pyfunction!(pr_close, m)?)?;
    m.add_function(wrap_pyfunction!(pr_approve, m)?)?;
    m.add_function(wrap_pyfunction!(pr_diff, m)?)?;
    m.add_function(wrap_pyfunction!(pr_comments, m)?)?;
    m.add_function(wrap_pyfunction!(pr_comment, m)?)?;
    m.add_function(wrap_pyfunction!(pr_checks, m)?)?;

    Ok(())
}
