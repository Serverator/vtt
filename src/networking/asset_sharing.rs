use crate::prelude::*;
use bevy::{asset::AssetPath, render::{render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureDimension, TextureFormat}}};
use lightyear::prelude::*;

pub trait SharedAssetExt {
    fn add_shared_asset<T: Sharable>(&mut self) -> &mut Self;
}

impl SharedAssetExt for App {
    fn add_shared_asset<T: Sharable>(&mut self) -> &mut Self {
        self.register_component::<SharedAsset<T>>(ChannelDirection::ServerToClient);
        self.register_type::<SharedAsset<T>>();

        self.init_resource::<SharedAssets<T>>();

        self.add_message::<RequestAssetMessage<T>>(ChannelDirection::ClientToServer);
        self.add_message::<SendAssetMessage<T>>(ChannelDirection::ServerToClient);

        self.add_systems(PreUpdate, send_requested_asset::<T>.after(MainSet::EmitEvents).run_if(in_state(server::NetworkingState::Started)));
        self.add_systems(PreUpdate, recieve_requested_asset::<T>.after(MainSet::EmitEvents).run_if(in_state(client::NetworkingState::Connected)));

        self
    }
}

#[derive(Serialize, Deserialize)]
pub struct SharableImage {
    pub data: Vec<u8>,
    pub size: UVec2,
    pub format: SharableFormat,
}

#[derive(Serialize, Deserialize)]
pub enum SharableFormat {
   R8Unorm,
   Rg8Unorm,
   Bgra8Unorm,
   Rgba8UnormSrgb,
   Bgra8UnormSrgb,
}

impl SharableImage {
    pub fn from_image(image: &Image) -> Option<Self> {
        let format = SharableFormat::from_bevy_format(image.texture_descriptor.format)?;
        let data = image.data.clone();
        let size = image.size();
        
        Some(SharableImage {
            data,
            size,
            format,
        })
    }

    pub fn to_image(&self) -> Image {
        let size = Extent3d {
            width: self.size.x,
            height: self.size.y,
            ..default()
        };

        Image::new(size, TextureDimension::D2, self.data.clone(), self.format.to_bevy_format(), RenderAssetUsages::RENDER_WORLD)
    }

    pub fn into_image(self) -> Image {
        let size = Extent3d {
            width: self.size.x,
            height: self.size.y,
            ..default()
        };

        Image::new(size, TextureDimension::D2, self.data, self.format.to_bevy_format(), RenderAssetUsages::RENDER_WORLD)
    }
}

impl SharableFormat {
    fn from_bevy_format(format: TextureFormat) -> Option<SharableFormat> {
        match format {
            TextureFormat::R8Unorm => Some(SharableFormat::R8Unorm),
            TextureFormat::Rg8Unorm => Some(SharableFormat::Rg8Unorm),
            TextureFormat::Bgra8Unorm => Some(SharableFormat::Bgra8Unorm),
            TextureFormat::Rgba8UnormSrgb => Some(SharableFormat::Rgba8UnormSrgb),
            TextureFormat::Bgra8UnormSrgb => Some(SharableFormat::Bgra8UnormSrgb),
            _ => None,
        }
    } 

    fn to_bevy_format(&self) -> TextureFormat {
        match self {
            SharableFormat::R8Unorm => TextureFormat::R8Unorm,
            SharableFormat::Rg8Unorm => TextureFormat::Rg8Unorm,
            SharableFormat::Bgra8Unorm => TextureFormat::Bgra8Unorm,
            SharableFormat::Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
            SharableFormat::Bgra8UnormSrgb => TextureFormat::Bgra8UnormSrgb,
        }
    }
}

// impl SharableImage {
//     pub fn into_image() -> Image {
//         image::load_from_memory_with_format(buf, image::ImageFormat::Bmp)
//         Image::from_dynamic(dyn_img, is_srgb, asset_usage)
//         Image::new(size, dimension, data, format, asset_usage)
//     }
// }


#[derive(Resource)]
pub struct SharedAssets<T: Asset> {
    pub id_to_handle: HashMap<Uuid, Handle<T>>,
    pub handle_to_id: HashMap<Handle<T>, Uuid>,
}

impl<T: Asset> SharedAssets<T> {
    pub fn load_shared<'a>(&mut self, asset_server: &AssetServer, path: impl Into<AssetPath<'a>>) -> (Handle<T>, Uuid) {
        let handle = asset_server.load(path);
        let uuid = Uuid::new_v4();
        self.id_to_handle.insert(uuid, handle.clone());
        self.handle_to_id.insert(handle.clone_weak(), uuid);

        (handle, uuid)
    }
}

// Derive macro for some reason refuses to impl Default
impl<T: Asset> Default for SharedAssets<T> {
    fn default() -> Self {
        Self {
            id_to_handle: HashMap::default(),
            handle_to_id: HashMap::default(),
        }
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RequestAssetMessage<T> {
    pub id: Uuid,
    _spooky: PhantomData<T>,
}

impl<T> RequestAssetMessage<T> {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            id: uuid,
            _spooky: PhantomData,
        }
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SendAssetMessage<T: Sharable> {
    pub id: Uuid,
    pub data: T::SharableType,
}

/// Converts data into sharable format
pub trait Sharable where Self: Sized + Asset + Typed, Self::SharableType: Message {
    type SharableType;

    fn to_sharable(&self) -> Option<Self::SharableType>;

    fn from_sharable(sharable: &Self::SharableType) -> Option<Self>;
}

impl Sharable for Image {
    type SharableType = SharableImage;
    
    fn to_sharable(&self) -> Option<Self::SharableType> {
        SharableImage::from_image(self)
    }
    
    fn from_sharable(sharable: &Self::SharableType) -> Option<Self> {
        Some(SharableImage::to_image(sharable))
    }
}

fn send_requested_asset<T: Sharable>(
    assets: Res<Assets<T>>,
    shared_assets: Res<SharedAssets<T>>,
    mut requests: EventReader<server::MessageEvent<RequestAssetMessage<T>>>,
    mut connection: ResMut<server::ConnectionManager>,
) {
    for request in requests.read() {
        let client = request.context;
        let asset_id = request.message.id;

        let Some(asset) = shared_assets.id_to_handle.get(&asset_id) else {
            info!("Asset of type '{:?}' with UUID '{asset_id}' is not shared" , T::type_ident());
            continue;
        };

        if let Some(asset) = assets.get(asset) {
            let Some(sharable) = asset.to_sharable() else {
                error!("Failed to convert {:?} to sharable type", T::type_ident());
                continue;
            };

            let message = SendAssetMessage::<T> {
                id: asset_id,
                data: sharable,
            };

            connection.send_message::<UnorderedReliable, _>(client, &message).unwrap();
        } else {
            info!("Asset of type '{:?}' with UUID '{asset_id}' not found" , T::type_ident())
        }
    }
}

fn recieve_requested_asset<T: Sharable>(
    mut assets: ResMut<Assets<T>>,
    mut recieved_asset: EventReader<client::MessageEvent<SendAssetMessage<T>>>,
) {
    for asset in recieved_asset.read() {
        let asset_id = asset.message.id;
        let Some(asset) = T::from_sharable(&asset.message.data) else {
            error!("Failed to convert {:?} from sharable type", T::type_ident());
            continue;
        };

        assets.insert(asset_id, asset);
    }
}
