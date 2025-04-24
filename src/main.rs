use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::App,
    prelude::*,
    text::FontSmoothing,
    window::{CursorGrabMode, PresentMode, PrimaryWindow},
};
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use bevy_rapier3d::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static VSYNC: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));

struct OverlayColor;

#[allow(dead_code)]
impl OverlayColor {
    const RED: Color = Color::srgb(1.0, 0.0, 0.0);
    const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: 20.0,
                        font: default(),
                        font_smoothing: FontSmoothing::AntiAliased,
                    },
                    text_color: OverlayColor::GREEN,
                    enabled: true,
                },
            },
            EguiPlugin,
        ))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, (setup, setup_camera))
        .add_systems(Update, (keybinds, game_ui, game_setting))
        .run();
}

// fn object_rigidbody() {}

fn game_ui(mut contexts: EguiContexts) {
    let mut vsync_status = VSYNC.lock().unwrap();
    egui::Window::new("Hello World").show(contexts.ctx_mut(), |ui| {
        ui.checkbox(&mut *vsync_status, "Vsync");
    });
}

fn game_setting(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = windows.single_mut();
    let vsync_status = VSYNC.lock().unwrap();

    if *vsync_status == true {
        window.present_mode = PresentMode::AutoVsync;
    } else {
        window.present_mode = PresentMode::AutoNoVsync;
    }
}

fn keybinds(
    key_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if key_input.just_pressed(KeyCode::ControlLeft) {
        lock_hide_cursor(windows);
    }
}

fn lock_hide_cursor(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = windows.single_mut();
    println!("Mouse Status: {}", window.cursor_options.visible);
    if window.cursor_options.visible == true {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    } else {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-1.0, 5.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ground_size = Vec3::new(10.0, 1.0, 9.0);

    // FPS Counter
    commands.spawn((Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(12.),
        left: Val::Px(12.),
        ..Default::default()
    },));

    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(ground_size.x, ground_size.y, ground_size.z))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Collider::cuboid(
            ground_size.x / 2.0,
            ground_size.y / 2.0,
            ground_size.z / 2.0,
        ),
        Transform::from_xyz(0.0, -2.0, 0.0),
    ));

    // Sphere
    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Restitution::coefficient(1.0),
        Transform::from_xyz(0.0, 8.0, 0.0),
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
    ));

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 3.0, 10.0),
    ));
}
