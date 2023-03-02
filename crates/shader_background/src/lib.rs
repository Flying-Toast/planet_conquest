use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<BackgroundMaterial>::default())
            .add_startup_system(create_bg)
            .add_system(update_bg.at_end());
    }
}

#[derive(AsBindGroup, Clone, TypeUuid)]
#[uuid = "1782c90e-b8ff-11ed-ab91-8c8caa6a5259"]
struct BackgroundMaterial {
    #[uniform(0)]
    base_color: Color,
    #[uniform(0)]
    noise_color: Color,
    #[uniform(0)]
    time: f32,
}

impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/background.wgsl".into()
    }
}

#[derive(Component)]
struct Background;

fn create_bg(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform: Transform::default().with_translation(Vec3::new(0., 0., -1.)),
            material: materials.add(BackgroundMaterial {
                base_color: Color::rgb(0.168, 0.094, 0.376),
                noise_color: Color::rgb(0.2, 0.243, 0.478),
                time: 0.,
            }),
            ..default()
        })
        .insert(Background);
}

fn update_bg(
    mut bg_query: Query<
        (&mut Transform, &Mesh2dHandle, &Handle<BackgroundMaterial>),
        (With<Background>, Without<Camera>),
    >,
    camera_query: Query<(&Transform, &Camera)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
    time: Res<Time>,
) {
    let (camera_transform, camera) = camera_query.single();
    let viewport_size = camera.logical_viewport_size().unwrap();

    let (mut bg_transform, Mesh2dHandle(mesh_handle), material_handle) = bg_query.single_mut();

    *meshes.get_mut(mesh_handle).unwrap() = Mesh::from(shape::Quad::new(viewport_size));
    bg_transform.translation = camera_transform.translation.xy().extend(-1.);

    materials.get_mut(material_handle).unwrap().time = time.raw_elapsed_seconds_wrapped();
}
