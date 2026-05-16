use std::path::Path;
use std::sync::Arc;

use core_infra::{
    FluentTranslator, SeaActiveMessageRepository, SeaChannelPatternRepository,
    SeaGuildSettingsRepository, SeaTaskRepository, SystemClock,
};
use discord_task::commands::{ConfigSetHandler, CountHandler, TaskStateHandler};
use discord_task::events::{
    AnyEventHandler, ChannelCreateHandler, ThreadCreateHandler, ThreadDeleteHandler,
};
use discord_task_infra::{
    TwilightActiveMessageService, TwilightChannelInspector, TwilightForumService,
    TwilightInteractionResponder, TwilightTaskNotifier,
};
use sea_orm::DatabaseConnection;
use twilight_model::id::marker::ApplicationMarker;
use twilight_model::id::Id;

pub type ConcreteAnyEventHandler = AnyEventHandler<
    SeaTaskRepository,
    SystemClock,
    SeaGuildSettingsRepository,
    SeaChannelPatternRepository,
    TwilightForumService,
>;

pub type ConcreteConfigSetHandler =
    ConfigSetHandler<SeaGuildSettingsRepository, SystemClock, TwilightInteractionResponder>;

pub type ConcreteTaskStateHandler =
    TaskStateHandler<SeaTaskRepository, SystemClock, TwilightInteractionResponder>;

pub type ConcreteCountHandler = CountHandler<
    SeaChannelPatternRepository,
    TwilightChannelInspector,
    TwilightInteractionResponder,
>;

pub type ConcreteTaskNotifier = TwilightTaskNotifier<FluentTranslator>;

pub struct ConcreteCommandHandlers {
    pub config_set: ConcreteConfigSetHandler,
    pub task_state: ConcreteTaskStateHandler,
    pub count: ConcreteCountHandler,
}

pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub clock: SystemClock,
    pub translator: Arc<FluentTranslator>,
    pub responder: TwilightInteractionResponder,
    pub forum_service: TwilightForumService,
    pub inspector: TwilightChannelInspector,
    pub notifier: ConcreteTaskNotifier,
    pub active_message_service: TwilightActiveMessageService,
}

pub fn build_app_state(
    db: Arc<DatabaseConnection>,
    http: Arc<twilight_http::Client>,
    application_id: Id<ApplicationMarker>,
    locales_dir: &Path,
) -> AppState {
    let translator =
        Arc::new(FluentTranslator::load(locales_dir).expect("failed to load locale files"));
    AppState {
        clock: SystemClock,
        translator: Arc::clone(&translator),
        responder: TwilightInteractionResponder::new(Arc::clone(&http), application_id),
        forum_service: TwilightForumService::new(Arc::clone(&http)),
        inspector: TwilightChannelInspector::new(Arc::clone(&http)),
        notifier: TwilightTaskNotifier::new(Arc::clone(&http), Arc::clone(&translator)),
        active_message_service: TwilightActiveMessageService::new(
            Arc::clone(&http),
            SeaActiveMessageRepository::new(Arc::clone(&db)),
        ),
        db,
    }
}

pub fn build_event_handlers(state: &AppState) -> Vec<ConcreteAnyEventHandler> {
    vec![
        AnyEventHandler::ThreadCreate(ThreadCreateHandler::new(
            SeaTaskRepository::new(Arc::clone(&state.db)),
            state.clock,
        )),
        AnyEventHandler::ThreadDelete(ThreadDeleteHandler::new(
            SeaTaskRepository::new(Arc::clone(&state.db)),
            state.clock,
        )),
        AnyEventHandler::ChannelCreate(ChannelCreateHandler::new(
            SeaGuildSettingsRepository::new(Arc::clone(&state.db)),
            SeaChannelPatternRepository::new(Arc::clone(&state.db)),
            state.forum_service.clone(),
        )),
    ]
}

pub fn build_command_handlers(state: &AppState) -> ConcreteCommandHandlers {
    ConcreteCommandHandlers {
        config_set: ConfigSetHandler::new(
            SeaGuildSettingsRepository::new(Arc::clone(&state.db)),
            state.clock,
            state.responder.clone(),
        ),
        task_state: TaskStateHandler::new(
            SeaTaskRepository::new(Arc::clone(&state.db)),
            state.clock,
            state.responder.clone(),
        ),
        count: CountHandler::new(
            SeaChannelPatternRepository::new(Arc::clone(&state.db)),
            state.inspector.clone(),
            state.responder.clone(),
        ),
    }
}
