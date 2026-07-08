use nannou::glam::Vec3;

#[allow(dead_code)]
#[derive(Clone)]
pub enum MaterialType {
    DIFFUSE,
    SPECULAR,
}
#[allow(dead_code)]
#[derive(Clone)]
pub struct Material {
    // 色
    pub emission: Option<Vec3>,
    // 反射方向
    pub reflection: Option<Vec3>,
    pub material_type: MaterialType,
}

#[allow(dead_code)]
impl Material {
    pub fn new(
        emission: Option<Vec3>,
        reflection: Option<Vec3>,
        material_type: Option<MaterialType>,
    ) -> Self {
        let material_type = match material_type {
            Some(t) => t,
            None => MaterialType::DIFFUSE,
        };
        Self {
            emission,
            reflection,
            material_type,
        }
    }

    pub fn color(&self) -> Vec3 {
        if let Some(emission) = self.emission {
            return emission;
        }
        if let Some(reflection) = self.reflection {
            return reflection;
        }
        Vec3::ZERO
    }
}
