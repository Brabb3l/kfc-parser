use std::rc::Rc;

use mod_loader::{Capability, Mod, ModManifest, ModRegistry};
use tracing::info;

use crate::{env::{AppFeatures, AppState}, lua::{FunctionArgs, LuaVM}, runner::state::{RunnerLocalState, RunnerState}};

mod state;

pub struct LuaModRunner {
    pub(crate) lua: LuaVM,
    state: Rc<RunnerState>,
}

impl LuaModRunner {

    pub fn new(
        context: AppState,
    ) -> mlua::Result<Self> {
        let lua = LuaVM::new()?;
        let state = Rc::new(RunnerState::default());

        lua.set_app_data(context);

        Ok(Self {
            lua,
            state,
        })
    }

    pub fn run(&self) -> mlua::Result<()> {
        let app_state = self.lua.app_data_ref::<AppState>().unwrap();

        for r#mod in self.state.mods().values() {
            let has_patch_capability = r#mod.info().capabilities.contains(&Capability::Patch);
            let has_export_capability = r#mod.info().capabilities.contains(&Capability::Export);
            let should_run =
                has_patch_capability && app_state.has_feature(AppFeatures::PATCH) ||
                has_export_capability && app_state.has_feature(AppFeatures::EXPORT);

            if !should_run {
                continue;
            }

            info!(
                mod_id = r#mod.info().id,
                mod_name = r#mod.info().name,
                "Running mod",
            );

            self.state.require(
                &format!("mods.{}", r#mod.info().id),
                &self.lua
            )?;
        }

        Ok(())
    }

    pub fn setup(
        &self,
        mod_registry: &ModRegistry,
    ) -> mlua::Result<()> {
        for r#mod in mod_registry.values() {
            self.setup_mod(r#mod)?;
        }

        Ok(())
    }

    fn setup_mod(
        &self,
        r#mod: &Mod,
    ) -> mlua::Result<()> {
        let env = self.create_mod_environment(r#mod.info())?;

        crate::env::register(
            &self.lua,
            &env,
            r#mod,
        )?;

        self.state.add_mod(
            r#mod.clone(),
            env,
        );

        Ok(())
    }

    fn create_mod_environment(
        &self,
        mod_info: &ModManifest,
    ) -> mlua::Result<mlua::Table> {
        let env = self.lua.create_base_environment()?;
        let local_state = RunnerLocalState::new(
            &mod_info.id,
            self.state.clone(),
        );

        env.raw_set(
            "require",
            self.lua.create_function(move |lua, args: FunctionArgs| {
                let id = args.get::<String>(0)?;

                local_state.require(&id, lua)
            })?
        )?;

        Ok(env)
    }


}
