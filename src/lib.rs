use bevy::prelude::*;

use bevy::ecs::schedule::StateData;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;

/// Plugin to add a simple splash-screen state
///
/// Create a new splash screen by providing the app
/// state during which to display it, and the next app
/// state to transition to. You can create as many
/// instances of this plugin as you need, if you want
/// multiple splash screens.
///
/// You are expected to populate the splash screen with
/// whatever you want to display, yourself. Just spawn
/// entities and insert one of the following components:
///  - [`SplashItemTimeout`]
///  - [`SplashItemFade`]
///
/// When the [`Timer`][bevy::time::Timer]s inside all
/// such components have completed (all splash screen
/// entities are finished displaying), a state transition
/// will be performed to the `next` state.
///
/// The splash screen is skippable by the user, by default.
/// Any of the following input events will cause the
/// state transition to be performed immediately:
///  - any keyboard keypress
///  - any mouse button press
///  - any gamepad button press
///  - any started touchscreen touch
///
/// To disable this behavior, set `skippable` to `false`.
///
/// If you would like to perform other background work
/// during your splash screen (such as loading assets,
/// etc.), consider using [`SplashProgressPlugin`]
/// instead (with the `iyes_progress` cargo feature).
pub struct SplashPlugin<S: StateData> {
    pub state: S,
    pub next: S,
    pub skippable: bool,
}

impl<S: StateData> SplashPlugin<S> {
    /// Create a new splash screen
    ///
    /// Will run in `state` and transition to `next`.
    pub fn new(state: S, next: S) -> Self {
        SplashPlugin {
            state,
            next,
            skippable: true,
        }
    }
}

/// Plugin to add a splash-screen based on `iyes_progress`
///
/// Create a new splash screen by providing the app state
/// during which to display it. The provided state must
/// be registered with `iyes_progress`, to have progress
/// tracking enabled for it.
///
/// (If you don't need progress tracking, use
/// [`SplashPlugin`] instead.)
///
/// This plugin will rely on `iyes_progress` to perform
/// the state transition. It will only happen when all of
/// your other progress-reporting systems have completed,
/// as well as the splash screen.
///
/// This allows you to perform and track other work (such
/// as loading assets, etc.) during your splash screen.
//
/// The plugin will report the state of the splash screen
/// as "hidden progress" to `iyes_progress`, so it can be
/// accounted together with your other progress-tracking
/// systems. The splash screen is considered "completed"
/// when either all items have timed out, or the user has
/// chosen to skip it.
///
/// In effect, if the user chooses to skip the splash screen,
/// but there is still other incomplete work going on, the
/// skip will be delayed until your backgound work completes.
///
/// This plugin will never trigger a state transition by
/// itself. Configure your next state in `iyes_progress`.
///
/// ---
///
/// You are expected to populate the splash screen with
/// whatever you want to display, yourself. Just spawn
/// entities and insert one of the following components:
///  - [`SplashItemTimeout`]
///  - [`SplashItemFade`]
///
/// When the [`Timer`][bevy::time::Timer]s inside all
/// such components have completed (all splash screen
/// entities are finished displaying), the splash screen
/// will be reported as "completed" to `iyes_progress`.
///
/// The splash screen is skippable by the user, by default.
/// Any of the following input events will cause the
/// splash screen to "complete" immediately:
///  - any keyboard keypress
///  - any mouse button press
///  - any gamepad button press
///  - any started touchscreen touch
///
/// To disable this behavior, set `skippable` to `false`.
pub struct SplashProgressPlugin<S: StateData> {
    pub state: S,
    pub skippable: bool,
}

impl<S: StateData> SplashProgressPlugin<S> {
    /// Create a new splash screen
    ///
    /// Will run in `state`.
    /// It must be a state with `iyes_progress` tracking.
    pub fn new(state: S) -> Self {
        SplashProgressPlugin {
            state,
            skippable: true,
        }
    }
}
#[cfg(feature = "iyes_loopless")]
impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppGlobalState::SplashIyes, splash_init_iyes);
        app.add_exit_system(AppGlobalState::SplashIyes, despawn_with_recursive::<SplashCleanup>);
        app.add_exit_system(AppGlobalState::SplashIyes, remove_resource::<SplashNext>);
        app.add_enter_system(AppGlobalState::SplashBevy, splash_init_bevy);
        app.add_exit_system(AppGlobalState::SplashBevy, despawn_with_recursive::<SplashCleanup>);
        app.add_exit_system(AppGlobalState::SplashBevy, remove_resource::<SplashNext>);
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(AppGlobalState::SplashIyes)
                .with_system(splash_skip)
                .with_system(splash_fade)
                .into()
        );
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(AppGlobalState::SplashBevy)
                .with_system(splash_skip)
                .with_system(splash_fade)
                .into()
        );
        app.add_exit_system(AppGlobalState::SplashBevy, remove_resource::<Splashes>);
        app.add_system_to_stage(CoreStage::PostUpdate, update_loading_pct.run_in_state(AppGlobalState::AssetsLoading));
    }
}

// fn update_loading_pct(
//     mut q: Query<&mut Text, With<LoadingPctText>>,
//     progress: Res<ProgressCounter>,
// ) {
//     let progress: f32 = progress.progress().into();
//     for mut txt in q.iter_mut() {
//         txt.sections[0].value = format!("{:.0}%", progress * 100.0);
//     }
// }

#[derive(Component)]
struct SplashCleanup;

struct SplashNext(AppGlobalState);

fn splash_init_iyes(
    mut commands: Commands,
    splashes: Res<Splashes>,
) {
    commands.insert_resource(SplashNext(AppGlobalState::SplashBevy));
    commands.spawn_bundle(Camera2dBundle::default())
        .insert(SplashCleanup);
    commands.spawn_bundle(SpriteBundle {
        texture: splashes.logo_iyeshead.clone(),
        transform: Transform::from_xyz(0.0, 75.0, 0.0),
        ..Default::default()
    }).insert(SplashCleanup)
    .insert(SplashFade::new(0.0, 0.0, 1.25, 1.5));
    commands.spawn_bundle(SpriteBundle {
        texture: splashes.logo_iyestext.clone(),
        transform: Transform::from_xyz(0.0, -175.0, 0.0),
        ..Default::default()
    }).insert(SplashCleanup)
    .insert(SplashFade::new(0.25, 0.75, 0.25, 1.75));
}

fn splash_init_bevy(
    mut commands: Commands,
    splashes: Res<Splashes>,
) {
    commands.insert_resource(SplashNext(AppGlobalState::MainMenu));
    commands.spawn_bundle(Camera2dBundle::default())
        .insert(SplashCleanup);
    commands.spawn_bundle(SpriteBundle {
        texture: splashes.logo_bevy.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    }).insert(SplashCleanup)
    .insert(SplashFade::new(0.0, 0.5, 1.0, 1.5));
}

#[derive(Component)]
struct SplashFade {
    timer_wait: Timer,
    timer_intro: Timer,
    timer_on: Timer,
    timer_fade: Timer,
}

impl SplashFade {
    fn new(wait: f32, intro: f32, on: f32, fade: f32) -> Self {
        Self {
            timer_wait: Timer::from_seconds(wait, false),
            timer_intro: Timer::from_seconds(intro, false),
            timer_on: Timer::from_seconds(on, false),
            timer_fade: Timer::from_seconds(fade, false),
        }
    }
}

fn splash_fade(
    mut q: Query<(&mut Sprite, &mut SplashFade)>,
    mut commands: Commands,
    t: Res<Time>,
    next: Res<SplashNext>,
) {
    let mut all_finished = true;
    let mut count = 0;
    for (mut sprite, mut fade) in q.iter_mut() {
        count += 1;
        if fade.timer_wait.duration().as_secs_f32() > 0.0 && !fade.timer_wait.finished() {
            fade.timer_wait.tick(t.delta());
            all_finished = false;
            sprite.color.set_a(0.0);
        } else if fade.timer_intro.duration().as_secs_f32() > 0.0 && !fade.timer_intro.finished() {
            fade.timer_intro.tick(t.delta());
            all_finished = false;
            let remain = fade.timer_intro.percent();
            sprite.color.set_a(remain);
        } else if !fade.timer_on.finished() {
            fade.timer_on.tick(t.delta());
            all_finished = false;
            sprite.color.set_a(1.0);
        } else if !fade.timer_fade.finished() {
            fade.timer_fade.tick(t.delta());
            all_finished = false;
            let remain = fade.timer_fade.percent_left();
            sprite.color.set_a(remain);
        }
    }
    if all_finished && count > 0 {
        commands.insert_resource(NextState(next.0));
    }
}

fn splash_skip(
    mut commands: Commands,
    mut kbd: EventReader<KeyboardInput>,
    mut mouse: EventReader<MouseButtonInput>,
    mut gamepad: EventReader<GamepadEvent>,
    mut touch: EventReader<TouchInput>,
) {
    use bevy::input::ButtonState;
    use bevy::input::touch::TouchPhase;

    let mut done = false;

    for ev in kbd.iter() {
        if let ButtonState::Pressed = ev.state {
            done = true;
        }
    }

    for ev in mouse.iter() {
        if let ButtonState::Pressed = ev.state {
            done = true;
        }
    }

    for ev in gamepad.iter() {
        if let GamepadEventType::ButtonChanged(_, _) = ev.event_type {
            done = true;
        }
    }

    for ev in touch.iter() {
        if let TouchPhase::Started = ev.phase {
            done = true;
        }
    }

    if done {
        commands.insert_resource(NextState(AppGlobalState::MainMenu));
    }
}
