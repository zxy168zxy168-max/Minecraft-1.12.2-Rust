use crate::net::minecraft::client::entity::EntityOtherClient::EntityOtherClient;
use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingPartTransform, LivingRenderInput,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LlamaPose {
    head: PartPose,
    body: PartPose,
    chestLeft: PartPose,
    chestRight: PartPose,
    legs: [PartPose; 4],
}

pub struct ModelLlama;
impl ModelLlama {
    pub fn pose(input: LivingRenderInput) -> LlamaPose {
        let head = PartPose {
            pivot: [0.0, 7.0, -6.0],
            rotation: [
                input.headPitch.to_radians(),
                (input.headYaw - input.bodyYaw).to_radians(),
                0.0,
            ],
        };
        let body = PartPose {
            pivot: [0.0, 5.0, 2.0],
            rotation: [std::f32::consts::FRAC_PI_2, 0.0, 0.0],
        };
        let swing = input.limbSwing * 0.6662;
        let amount = 1.4 * input.limbSwingAmount;
        let leg1 = PartPose {
            pivot: [-3.5, 10.0, 6.0],
            rotation: [(swing).cos() * amount, 0.0, 0.0],
        };
        let leg2 = PartPose {
            pivot: [3.5, 10.0, 6.0],
            rotation: [(swing + std::f32::consts::PI).cos() * amount, 0.0, 0.0],
        };
        let leg3 = PartPose {
            pivot: [-3.5, 10.0, -5.0],
            rotation: [(swing + std::f32::consts::PI).cos() * amount, 0.0, 0.0],
        };
        let leg4 = PartPose {
            pivot: [3.5, 10.0, -5.0],
            rotation: [(swing).cos() * amount, 0.0, 0.0],
        };
        LlamaPose {
            head,
            body,
            chestLeft: PartPose {
                pivot: [-8.5, 3.0, 3.0],
                rotation: [0.0, std::f32::consts::FRAC_PI_2, 0.0],
            },
            chestRight: PartPose {
                pivot: [5.5, 3.0, 3.0],
                rotation: [0.0, std::f32::consts::FRAC_PI_2, 0.0],
            },
            legs: [leg1, leg2, leg3, leg4],
        }
    }

    pub fn boxes(
        pose: LlamaPose,
        input: LivingRenderInput,
        entity: &EntityOtherClient,
        delta: f32,
    ) -> Vec<LivingModelBox> {
        let headTransform = input.child.then_some(LivingPartTransform {
            scale: [0.71428573, 0.64935064, 0.7936508],
            translation: [0.0, 21.0 * 0.0625, 0.22],
        });
        let bodyTransform = input.child.then_some(LivingPartTransform {
            scale: [0.625, 0.45454544, 0.45454544],
            translation: [0.0, 33.0 * 0.0625, 0.0],
        });
        let legTransform = input.child.then_some(LivingPartTransform {
            scale: [0.45454544, 0.41322312, 0.45454544],
            translation: [0.0, 33.0 * 0.0625, 0.0],
        });
        let mut boxes = Vec::with_capacity(11);
        let mut push = |texture, origin, size, partPose, transform| {
            let mut b = model_box(
                texture,
                origin,
                size,
                delta,
                false,
                partPose,
                LivingModelGroup::Body,
            );
            b.childTransform = transform;
            boxes.push(b);
        };
        push(
            [0, 0],
            [-2.0, -14.0, -10.0],
            [4, 4, 9],
            pose.head,
            headTransform,
        );
        push(
            [0, 14],
            [-4.0, -16.0, -6.0],
            [8, 18, 6],
            pose.head,
            headTransform,
        );
        push(
            [17, 0],
            [-4.0, -19.0, -4.0],
            [3, 3, 2],
            pose.head,
            headTransform,
        );
        push(
            [17, 0],
            [1.0, -19.0, -4.0],
            [3, 3, 2],
            pose.head,
            headTransform,
        );
        push(
            [29, 0],
            [-6.0, -10.0, -7.0],
            [12, 18, 10],
            pose.body,
            bodyTransform,
        );
        for leg in pose.legs {
            push([29, 29], [-2.0, 0.0, -2.0], [4, 14, 4], leg, legTransform);
        }
        if !input.child && entity.horseChested() {
            push([45, 28], [-3.0, 0.0, 0.0], [8, 8, 3], pose.chestLeft, None);
            push([45, 41], [-3.0, 0.0, 0.0], [8, 8, 3], pose.chestRight, None);
        }
        boxes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::entity::EntityOtherClient::{
        ClientEntityKind, MobEntityType,
    };
    use crate::net::minecraft::client::renderer::entity::RenderLivingBase::RenderLivingBase;

    #[test]
    fn llama_child_uses_three_distinct_source_scales() {
        let mut entity = EntityOtherClient::new(
            1,
            None,
            ClientEntityKind::Mob {
                entityType: MobEntityType::fromId(103).unwrap(),
            },
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        entity.applyMetadata([(
            12,
            crate::net::minecraft::network::datasync::DataSerializers::DataValue::Boolean(true),
        )]);
        let input = RenderLivingBase::renderInput(&entity, 0.0, 1.0);
        let boxes = ModelLlama::boxes(ModelLlama::pose(input), input, &entity, 0.0);
        let head = boxes[0].childTransform.unwrap().scale;
        let body = boxes[4].childTransform.unwrap().scale;
        let legs = boxes[5].childTransform.unwrap().scale;
        assert_ne!(head, body);
        assert_ne!(body, legs);
    }
}
