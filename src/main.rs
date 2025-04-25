use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    input::mouse::MouseMotion,
    prelude::{App, *},
    text::FontSmoothing,
    window::{CursorGrabMode, PresentMode, PrimaryWindow},
};
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use bevy_rapier3d::prelude::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static VSYNC: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static BALL_COUNTER: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0));
static MOUSE_SENSITIVITY: Lazy<Mutex<f32>> = Lazy::new(|| Mutex::new(0.1));
static GROUND_SIZE: Lazy<Mutex<Vec3>> = Lazy::new(|| {
    Mutex::new(Vec3 {
        x: 20.0,
        y: 1.0,
        z: 15.0,
    })
});

struct OverlayColor;

#[derive(Component)]
struct RotataCamera;

#[derive(Resource, Default)]
struct PreviousGroundSize(Vec3);

#[derive(Component)]
struct Ground;

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
        .insert_resource(PreviousGroundSize(Vec3::ZERO))
        .add_systems(Startup, (setup, setup_camera))
        .add_systems(
            Update,
            (keybinds, game_ui, game_setting, ground_change_detector),
        )
        .add_systems(Update, (mouse_free_look, mouse_movement))
        .run();
}

fn mouse_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    timer: Res<Time>,
    mut query: Query<&mut Transform, With<RotataCamera>>,
) {
    let mut transform = query.single_mut();
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += transform.forward().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction += transform.back().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction += transform.left().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += transform.right().as_vec3();
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
        let speed = 5.0;
        transform.translation += direction * speed * timer.delta_secs();
    }
}

fn mouse_free_look(
    mut cam: Query<&mut Transform, With<RotataCamera>>,
    timer: Res<Time>,
    mut evr_mouse_motion: EventReader<MouseMotion>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let window = windows.single_mut();

    if window.cursor_options.visible == false
    // && window.cursor_options.grab_mode == CursorGrabMode::Confined
    {
        let mouse_sensitivity = MOUSE_SENSITIVITY.lock().unwrap();
        let mut transform = cam.single_mut();

        for event in evr_mouse_motion.read() {
            let delta = event.delta * *mouse_sensitivity * timer.delta_secs();

            let yaw = Quat::from_rotation_y(-delta.x);
            let pitch = Quat::from_rotation_x(-delta.y);

            transform.rotation = yaw * transform.rotation.normalize();
            transform.rotation = transform.rotation.normalize() * pitch;
        }
    }
}

fn game_ui(mut contexts: EguiContexts) {
    let mut vsync_status = VSYNC.lock().unwrap();
    let mut ball_counter = BALL_COUNTER.lock().unwrap();
    let mut mouse_sensitivity = MOUSE_SENSITIVITY.lock().unwrap();
    let mut ground_size = GROUND_SIZE.lock().unwrap();

    let min_ball: u16 = 0;
    let max_ball: u16 = 100;

    let min_sensi: f32 = 0.1;
    let max_sensi: f32 = 1.0;

    egui::Window::new("Settings")
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.checkbox(&mut *vsync_status, "Vsync");

            ui.add(egui::Label::new("Ball Counter"));
            ui.add(egui::Slider::new(&mut *ball_counter, min_ball..=max_ball));

            ui.add(egui::Label::new("Mouse Sensitivity"));
            ui.add(egui::Slider::new(
                &mut *mouse_sensitivity,
                min_sensi..=max_sensi,
            ));
        });

    egui::Window::new("Ground Size")
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.add(egui::Label::new("X-Axis"));
            ui.add(egui::Slider::new(&mut ground_size.x, 10.0..=100.0));
            ui.add(egui::Label::new("Y-Axis"));
            ui.add(egui::Slider::new(&mut ground_size.y, 0.5..=2.0));
            ui.add(egui::Label::new("Z-Axis"));
            ui.add(egui::Slider::new(&mut ground_size.z, 10.0..=100.0));
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
        window.cursor_options.grab_mode = CursorGrabMode::Confined;
        window.cursor_options.hit_test = false;
    } else {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.hit_test = true;
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        RotataCamera,
        Camera3d::default(),
        Transform::from_xyz(-1.0, 10.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let ground_size = GROUND_SIZE.lock().unwrap();
    commands.spawn((
        Ground,
        Mesh3d(meshes.add(Cuboid::new(ground_size.x, ground_size.y, ground_size.z))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Collider::cuboid(
            ground_size.x / 2.0,
            ground_size.y / 2.0,
            ground_size.z / 2.0,
        ),
        Transform::from_xyz(0.0, -2.0, 0.0),
    ));
}

fn ground_change_detector(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut prev: ResMut<PreviousGroundSize>,
    query: Query<Entity, With<Ground>>,
) {
    let current = *GROUND_SIZE.lock().unwrap();

    if current != prev.0 {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        spawn_ground(&mut commands, &mut meshes, &mut materials);

        prev.0 = current;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // FPS Counter
    commands.spawn((Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(12.),
        left: Val::Px(12.),
        ..Default::default()
    },));

    commands.insert_resource(PreviousGroundSize(Vec3::ZERO));
    // Ground
    spawn_ground(&mut commands, &mut meshes, &mut materials);

    // commands.spawn((
    //     Mesh3d(meshes.add(Sphere::new(3.0))),
    //     MeshMaterial3d(materials.add(Color::WHITE)),
    //     Collider::ball(3.0),
    //     Transform::from_xyz(0.0, 2.0, 0.0),
    // ));

    // Sphere
    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Restitution::coefficient(1.0),
        Transform::from_xyz(0.0, 15.0, 0.0),
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
