mod api;
mod command;
mod config;
mod utils;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command(rename_all = "snake_case")]
async fn open_workspace_command(handle: tauri::AppHandle, id: &str) -> Result<bool, ()> {
    match command::workspace::open_workspace(handle, id).await {
        Ok(result) => Ok(result),
        Err(_) => Err(()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_workspace_command,
            command::app_config::read_app_config_command,
            command::app_config::delete_workspace_from_app_config_command,
            command::workspace::create_workspace_command,
            command::workspace::load_workspace_merge_info_command,
            command::workspace::update_workspace_details_command,
            command::workspace::load_workspaces_command,
            command::resource_provider::setup_app_command,
            command::stack::load_stacks_command,
            command::stack::delete_stack_command,
            command::stack::import_stack_command,
            command::stack::load_stack_command,
            command::stack::load_template_summary_command,
            command::stack::get_stack_resource_list_command,
            command::stack::get_stack_resource_properties_command,
            command::stack::get_stack_parameters_command,
            command::stack::get_stack_outputs_command,
            command::stack::get_all_stack_outputs_command,
            command::stack::update_stack_meta_command,
            command::stack::update_stack_detail_command,
            command::stack::get_stack_resource_properties_reasons_command,
            command::stack::load_parameter_and_resource_list_command,
            command::cloudformation_schema::get_cloudformation_schema_command,
            command::cloudformation_schema::get_aws_service_list_command,
            command::manual_management_resource::new_manual_management_resource_command,
            command::manual_management_resource::get_manual_management_resource_list_command,
            command::manual_management_resource::load_manual_management_resource_summary_command,
            command::manual_management_resource::get_manual_resource_properties_command,
            command::manual_management_resource::get_manual_resource_reasons_command,
            command::manual_management_resource::update_manual_resource_meta_command,
            command::manual_management_resource::update_manual_resource_properties_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
