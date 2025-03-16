use std::path::PathBuf;

use anyhow::Result;
use windows::Win32::Foundation::{VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::TaskScheduler::{
    IExecAction, ITaskDefinition, ITaskFolder, ITaskService, ITaskSettings, TASK_ACTION_EXEC,
    TASK_CREATE_OR_UPDATE, TASK_LOGON_INTERACTIVE_TOKEN, TASK_RUNLEVEL_HIGHEST, TASK_TRIGGER_LOGON,
    TaskScheduler,
};
use windows::Win32::System::Variant::VARIANT;
use windows::core::{BSTR, ComInterface};

pub fn create_autostart_task(executable_file_path: &PathBuf) -> Result<()> {
    unsafe {
        // Create an instance of the Task Scheduler service.
        let task_service: ITaskService =
            CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;

        // Connect to the task service.
        task_service.Connect(
            VARIANT::default(),
            VARIANT::default(),
            VARIANT::default(),
            VARIANT::default(),
        )?;

        // Get the root task folder.
        let root_folder: ITaskFolder = task_service
            .GetFolder(&BSTR::from("\\"))
            .expect("Failed to get the root folder");

        let task_folder = match root_folder.GetFolder(&BSTR::from("\\AudioSwitch")) {
            Ok(folder) => folder,
            Err(_) => root_folder.CreateFolder(&BSTR::from("AudioSwitch"), VARIANT::default())?,
        };

        // Create a new task definition.
        let task_definition: ITaskDefinition = task_service.NewTask(0)?;

        // Set registration info (for example, the author).
        let reg_info = task_definition.RegistrationInfo()?;
        reg_info.SetAuthor(&BSTR::from("AudioSwitch"))?;

        // Set the principal to run with highest privileges.
        let principal = task_definition.Principal()?;
        principal.SetRunLevel(TASK_RUNLEVEL_HIGHEST)?;

        // Create a boot trigger.
        let trigger_collection = task_definition.Triggers()?;
        let _boot_trigger = trigger_collection.Create(TASK_TRIGGER_LOGON)?;

        // Create an action to execute your program.
        let action_collection = task_definition.Actions()?;
        let action = action_collection.Create(TASK_ACTION_EXEC)?;
        let exec_action: IExecAction = action.cast()?;
        exec_action.SetPath(&BSTR::from(
            executable_file_path
                .to_str()
                .expect("Invalid executable path."),
        ))?;

        let task_settings: ITaskSettings = task_definition.Settings()?;

        task_settings.SetDisallowStartIfOnBatteries(VARIANT_FALSE)?;
        task_settings.SetStartWhenAvailable(VARIANT_TRUE)?;

        // Register the task in the root folder.
        let _registered_task = task_folder.RegisterTaskDefinition(
            &BSTR::from("AudioSwitchAutoStart"), // Task name
            &task_definition,
            TASK_CREATE_OR_UPDATE.0,
            VARIANT::default(), // UserId (not needed here)
            VARIANT::default(), // Password (not needed)
            TASK_LOGON_INTERACTIVE_TOKEN,
            VARIANT::default(), // No additional XML
        )?;
    }
    Ok(())
}

pub fn delete_task() -> Result<()> {
    unsafe {
        // Create an instance of the Task Scheduler service.
        let task_service: ITaskService =
            CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;

        // Connect to the task service.
        task_service.Connect(
            VARIANT::default(),
            VARIANT::default(),
            VARIANT::default(),
            VARIANT::default(),
        )?;

        // Retrieve the root task folder.
        let root_folder: ITaskFolder = task_service.GetFolder(&BSTR::from("\\AudioSwitch"))?;

        // Delete the specified task.
        root_folder.DeleteTask(&BSTR::from("AudioSwitchAutoStart"), 0)?;
    }
    Ok(())
}
