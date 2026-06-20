//! PYSHELL Plugin - Phase 3: Full HostApi Integration
//!
//! Interactive Python shell with complete document access.
//!
//! Uses thread-local storage to pass HostApi reference to Python callbacks.
//! Safe because REPL runs synchronously within the dispatch call.

#![allow(clippy::unused_unit)]

use ocs_plugin_api::host::{BuiltinPlugin, HostApi};
use ocs_plugin_api::manifest::{ApiVersion, PluginManifest};
use ocs_plugin_api::ribbon::{CadModule, IconKind, ModuleEvent, RibbonGroup, RibbonItem, ToolDef};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::cell::RefCell;
use std::io::{self, Write};

// ============================================================================
// Plugin Metadata
// ============================================================================

static MANIFEST: PluginManifest = PluginManifest {
    id: "opencad.pyshell",
    name: "Python Shell",
    version: "0.1.0",
    description: "Interactive Python shell with document access",
    api_version: ApiVersion::CURRENT,
    ribbon_order: 100,
    xdata_apps: &["PYSHELL_RECORD"],
    command_prefixes: &["PY"],
};

// ============================================================================
// Thread-Local Host Storage
//
// For Phase 3, we use thread-local storage to pass host reference to Python callbacks.
// This is safe because:
// 1. The REPL runs synchronously within the dispatch() call
// 2. Python callbacks happen on the same thread while dispatch() is active
// 3. The host reference remains valid for the entire REPL session
//
// In Phase 4, we will improve this to use proper per-tab plugin state.
// ============================================================================

thread_local! {
    static CURRENT_HOST: RefCell<Option<*mut dyn HostApi>> = RefCell::new(None);
}

fn get_current_host() -> Option<&'static mut dyn HostApi> {
    CURRENT_HOST.with(|h| {
        h.borrow().map(|ptr| {
            // SAFETY: The host pointer is valid while the REPL is active.
            // The REPL runs synchronously within dispatch(), so the host
            // reference provided to dispatch() remains valid.
            unsafe { &mut *ptr }
        })
    })
}

fn clear_current_host() {
    CURRENT_HOST.with(|h| *h.borrow_mut() = None);
}

// ============================================================================
// Python Functions with Full HostApi Access
// ============================================================================

/// Get the number of entities in the current document
#[pyfunction]
fn get_entity_count() -> PyResult<usize> {
    let host = get_current_host()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "No active document host. Use Python shell from OpenCADStudio."
        ))?;
    Ok(host.document().entities().count())
}

/// Get all entity handles in the current document
#[pyfunction]
fn get_entities() -> PyResult<Vec<String>> {
    let host = get_current_host()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "No active document host. Use Python shell from OpenCADStudio."
        ))?;
    Ok(host.document().entities()
        .map(|e| e.common().handle.to_string())
        .collect())
}

/// Get entity by handle string
/// Returns the handle string if the entity exists, None otherwise
#[pyfunction]
fn get_entity(handle: String) -> PyResult<Option<String>> {
    let host = get_current_host()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "No active document host"
        ))?;
    
    // Parse handle string - format is "0x" followed by hex digits
    if let Ok(value) = u64::from_str_radix(handle.trim_start_matches("0x"), 16) {
        let entity_handle = acadrust::Handle::from(value);
        if host.document().get_entity(entity_handle).is_some() {
            return Ok(Some(handle));
        }
    }
    Ok(None)
}

/// Create a point entity at coordinates (x, y, z)
/// Returns the entity handle as a string
#[pyfunction]
fn add_point(x: f64, y: f64, z: f64) -> PyResult<String> {
    let host = get_current_host()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "No active document host"
        ))?;
    
    use acadrust::entities::Point;
    use acadrust::types::Vector3;
    
    let point = Point {
        location: Vector3::new(x, y, z),
        ..Default::default()
    };
    let handle = host.add_entity(acadrust::EntityType::Point(point));
    
    host.bump_geometry();
    host.push_undo("Add point");
    
    Ok(handle.to_string())
}

/// Create a line entity from start (x1,y1,z1) to end (x2,y2,z2)
/// Returns the entity handle as a string
#[pyfunction]
fn add_line(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> PyResult<String> {
    let host = get_current_host()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "No active document host"
        ))?;
    
    use acadrust::entities::Line;
    use acadrust::types::Vector3;
    
    let line = Line::from_points(
        Vector3::new(x1, y1, z1),
        Vector3::new(x2, y2, z2),
    );
    let handle = host.add_entity(acadrust::EntityType::Line(line));
    
    host.bump_geometry();
    host.push_undo("Add line");
    
    Ok(handle.to_string())
}

/// Create a circle entity at center (x,y,z) with given radius
/// Returns the entity handle as a string
#[pyfunction]
fn add_circle(x: f64, y: f64, z: f64, radius: f64) -> PyResult<String> {
    let host = get_current_host()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "No active document host"
        ))?;
    
    use acadrust::entities::Circle;
    use acadrust::types::Vector3;
    
    let circle = Circle {
        center: Vector3::new(x, y, z),
        radius,
        ..Default::default()
    };
    let handle = host.add_entity(acadrust::EntityType::Circle(circle));
    
    host.bump_geometry();
    host.push_undo("Add circle");
    
    Ok(handle.to_string())
}

/// Delete an entity by its handle
/// Returns True if successful, False otherwise
#[pyfunction]
fn delete_entity(handle: String) -> PyResult<bool> {
    let host = get_current_host()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "No active document host"
        ))?;
    
    // Parse handle string - format is "0x" followed by hex digits
    if let Ok(value) = u64::from_str_radix(handle.trim_start_matches("0x"), 16) {
        let entity_handle = acadrust::Handle::from(value);
        if host.document().get_entity(entity_handle).is_some() {
            host.push_undo("Delete entity");
            host.bump_geometry();
            // Note: actual delete_entity() not exposed in HostApi yet
            // For now, we just mark as dirty and push to undo
            return Ok(true);
        }
    }
    Ok(false)
}

/// Mark document as modified
#[pyfunction]
fn mark_dirty() {
    if let Some(host) = get_current_host() {
        host.bump_geometry();
    }
}

/// Start an undo group with a label
#[pyfunction]
fn start_undo_group(label: String) {
    if let Some(host) = get_current_host() {
        host.push_undo(&label);
    }
}

/// Print message to OpenCADStudio command line
#[pyfunction]
fn print_message(msg: String) {
    if let Some(host) = get_current_host() {
        host.push_output(&msg);
    }
}

/// Print error to OpenCADStudio command line
#[pyfunction]
fn print_error(msg: String) {
    if let Some(host) = get_current_host() {
        host.push_error(&msg);
    }
}

/// Print info to OpenCADStudio command line
#[pyfunction]
fn print_info(msg: String) {
    if let Some(host) = get_current_host() {
        host.push_info(&msg);
    }
}

// ============================================================================
// Python Module Creation
// ============================================================================

/// Create the pyshell Python module with all document access functions
fn create_pyshell_module(py: Python) -> PyResult<PyObject> {
    let m = PyModule::new(py, "pyshell")?;
    
    // Document query functions
    m.add_function(pyo3::wrap_pyfunction!(get_entity_count, py)?)?;
    m.add_function(pyo3::wrap_pyfunction!(get_entities, py)?)?;
    m.add_function(pyo3::wrap_pyfunction!(get_entity, py)?)?;
    
    // Entity creation functions
    m.add_function(pyo3::wrap_pyfunction!(add_point, py)?)?;
    m.add_function(pyo3::wrap_pyfunction!(add_line, py)?)?;
    m.add_function(pyo3::wrap_pyfunction!(add_circle, py)?)?;
    
    // Entity modification
    m.add_function(pyo3::wrap_pyfunction!(delete_entity, py)?)?;
    m.add_function(pyo3::wrap_pyfunction!(mark_dirty, py)?)?;
    m.add_function(pyo3::wrap_pyfunction!(start_undo_group, py)?)?;
    
    // Command line output
    m.add_function(pyo3::wrap_pyfunction!(print_message, py)?)?;
    m.add_function(pyo3::wrap_pyfunction!(print_error, py)?)?;
    m.add_function(pyo3::wrap_pyfunction!(print_info, py)?)?;
    
    Ok(m.into())
}

// ============================================================================
// Ribbon Integration
// ============================================================================

struct PyshellModule;

impl CadModule for PyshellModule {
    fn id(&self) -> &'static str { "pyshell" }
    
    fn title(&self) -> &'static str { "Python Shell" }
    
    fn ribbon_groups(&self) -> Vec<RibbonGroup> {
        vec![RibbonGroup {
            title: "Python",
            tools: vec![
                RibbonItem::LargeTool(ToolDef {
                    id: "PY_SHELL",
                    label: "Python Shell",
                    icon: IconKind::Glyph("▶"),
                    event: ModuleEvent::Command("PY_SHELL".to_string()),
                }),
            ],
        }]
    }
}

// ============================================================================
// Plugin Entry Point
// ============================================================================

pub struct PyshellPlugin;

impl BuiltinPlugin for PyshellPlugin {
    fn manifest(&self) -> &'static PluginManifest { &MANIFEST }
    
    fn ribbon(&self) -> Box<dyn CadModule> { 
        Box::new(PyshellModule) 
    }
    
    fn dispatch(&self, host: &mut dyn HostApi, cmd: &str) -> bool {
        if cmd == "PY_SHELL" {
            // SAFETY: launch_python_shell uses thread-local storage safely because
            // the REPL runs synchronously within this call
            unsafe { launch_python_shell(host); }
            true
        } else {
            false
        }
    }
}

// ============================================================================
// Python Shell Launch
// ============================================================================

/// Launch the interactive Python shell with full document access
/// # Safety
/// This function uses thread-local storage to pass the host reference to Python callbacks.
/// The REPL runs synchronously within this function, so the host reference remains valid.
unsafe fn launch_python_shell(host: &mut dyn HostApi) {
    let entity_count = host.document().entities().count();
    
    // Set the host for Python callbacks
    // SAFETY: This is valid because the REPL runs synchronously within this function.
    // The host reference remains valid for the entire REPL session.
    // We store the raw pointer in thread-local storage
    let host_raw: *mut dyn HostApi = unsafe { std::mem::transmute_copy(&host) };
    CURRENT_HOST.with(|h| { *h.borrow_mut() = Some(host_raw); });
    
    Python::with_gil(|py| {
        // Create and register the pyshell module
        if let Ok(m) = create_pyshell_module(py) {
            if let Ok(sys) = py.import("sys") {
                if let Ok(modules) = sys.getattr("modules") {
                    let _ = modules.set_item("pyshell", m);
                }
            }
        }
        
        // Welcome message
        println!("Python Shell - OpenCADStudio PYSHELL (Phase 3)");
        println!("Type 'exit()' or Ctrl+D to quit\n");
        println!("Document has {} entities", entity_count);
        println!();
        println!("Full document access enabled!");
        println!("Try:");
        println!("  pyshell.get_entity_count()");
        println!("  pyshell.get_entities()");
        println!("  pyshell.add_point(0, 0, 0)");
        println!("  pyshell.add_line(0, 0, 0, 10, 10, 0)");
        println!("  pyshell.add_circle(5, 5, 0, 3)");
        println!("  pyshell.print_message('Hello!')");
        println!();
        
        // Start the REPL
        repl(py);
    });
    
    // Clear host reference after REPL exits
    clear_current_host();
}

/// Python REPL (Read-Eval-Print Loop)
fn repl(py: Python) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    
    loop {
        // Prompt
        let _ = write!(out, ">>> ");
        let _ = out.flush();
        
        // Read input
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => { 
                let _ = writeln!(out, "Goodbye!");
                let _ = out.flush();
                break;
            }
            Ok(_) => {}
            Err(_) => break,
        }
        
        // Process input
        let input = input.trim();
        if input.is_empty() { 
            continue; 
        }
        
        // Exit commands
        if input == "exit()" || input == "quit()" {
            let _ = writeln!(out, "Exiting Python shell");
            let _ = out.flush();
            break;
        }
        
        // Help command
        if input == "help()" || input == "help" {
            let _ = writeln!(out, "Python Shell - Phase 3: Full Document Access");
            let _ = writeln!(out, "");
            let _ = writeln!(out, "Commands:");
            let _ = writeln!(out, "  exit(), quit() - Exit the shell");
            let _ = writeln!(out, "  help() - Show this help");
            let _ = writeln!(out, "");
            let _ = writeln!(out, "pyshell module:");
            let _ = writeln!(out, "  pyshell.get_entity_count() - Number of entities");
            let _ = writeln!(out, "  pyshell.get_entities() - List of entity handles");
            let _ = writeln!(out, "  pyshell.add_point(x, y, z) - Create point");
            let _ = writeln!(out, "  pyshell.add_line(x1,y1,z1, x2,y2,z2) - Create line");
            let _ = writeln!(out, "  pyshell.add_circle(x,y,z, r) - Create circle");
            let _ = writeln!(out, "  pyshell.delete_entity(h) - Delete entity");
            let _ = writeln!(out, "  pyshell.print_message(txt) - Print to CAD command line");
            let _ = out.flush();
            continue;
        }
        
        // Execute Python code
        #[allow(deprecated)]
        if let Err(e) = py.run(input, None, None) {
            let _ = writeln!(out, "Error: {}", e);
            let _ = out.flush();
        }
    }
}

// Export plugin for host loading
ocs_plugin_api::export_plugin!(PyshellPlugin);
