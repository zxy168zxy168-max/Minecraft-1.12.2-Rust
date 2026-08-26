/// Rust semantic port of OptiFine 1.12.2
/// `net.optifine.shaders.ClippingHelperShadow` (HD_U_C6).
///
/// The helper starts from the ordinary camera frustum and extrudes its
/// silhouette planes along the active sun/moon direction. This is the exact
/// culling volume used by `ShadersRender.renderShadowMap`; it is intentionally
/// separate from the shadow projection itself.
#[derive(Debug, Clone)]
pub struct ClippingHelperShadow {
    shadowClipPlanes: Vec<[f32; 4]>,
}

impl ClippingHelperShadow {
    /// `projection` and `modelView` are column-major OpenGL matrices. The
    /// model-view must be camera-relative when callers also subtract the
    /// camera position in `isBoxInFrustum`, matching MCP `Frustum#setPosition`.
    pub fn fromMatrices(
        projection: [f32; 16],
        modelView: [f32; 16],
        shadowLightPositionVector: [f32; 4],
    ) -> Self {
        let clipping = multiply4(projection, modelView);
        let frustum = [
            assignPlane(
                clipping[3] - clipping[0],
                clipping[7] - clipping[4],
                clipping[11] - clipping[8],
                clipping[15] - clipping[12],
            ),
            assignPlane(
                clipping[3] + clipping[0],
                clipping[7] + clipping[4],
                clipping[11] + clipping[8],
                clipping[15] + clipping[12],
            ),
            assignPlane(
                clipping[3] + clipping[1],
                clipping[7] + clipping[5],
                clipping[11] + clipping[9],
                clipping[15] + clipping[13],
            ),
            assignPlane(
                clipping[3] - clipping[1],
                clipping[7] - clipping[5],
                clipping[11] - clipping[9],
                clipping[15] - clipping[13],
            ),
            assignPlane(
                clipping[3] - clipping[2],
                clipping[7] - clipping[6],
                clipping[11] - clipping[10],
                clipping[15] - clipping[14],
            ),
            assignPlane(
                clipping[3] + clipping[2],
                clipping[7] + clipping[6],
                clipping[11] + clipping[10],
                clipping[15] + clipping[14],
            ),
        ];
        let dots = frustum.map(|plane| dot3(plane, shadowLightPositionVector));
        let mut shadowClipPlanes = Vec::with_capacity(10);

        for positiveIndex in 0..6 {
            let positiveDot = dots[positiveIndex];
            if positiveDot < 0.0 {
                continue;
            }
            let positivePlane = frustum[positiveIndex];
            shadowClipPlanes.push(positivePlane);
            if positiveDot <= 0.0 {
                continue;
            }

            for negativeIndex in adjacentPlaneIndices(positiveIndex) {
                if dots[negativeIndex] < 0.0 {
                    shadowClipPlanes.push(makeShadowPlane(
                        positivePlane,
                        frustum[negativeIndex],
                        shadowLightPositionVector,
                    ));
                }
            }
        }

        Self { shadowClipPlanes }
    }

    /// Equivalent to MCP `Frustum` wrapping `ClippingHelperShadow`: the box is
    /// supplied in world coordinates and translated by the camera position
    /// before testing the camera-relative planes.
    #[allow(clippy::too_many_arguments)]
    pub fn isBoxInFrustum(
        &self,
        x1: f64,
        y1: f64,
        z1: f64,
        x2: f64,
        y2: f64,
        z2: f64,
        cameraPosition: [f32; 3],
    ) -> bool {
        let x1 = x1 - cameraPosition[0] as f64;
        let y1 = y1 - cameraPosition[1] as f64;
        let z1 = z1 - cameraPosition[2] as f64;
        let x2 = x2 - cameraPosition[0] as f64;
        let y2 = y2 - cameraPosition[1] as f64;
        let z2 = z2 - cameraPosition[2] as f64;

        self.shadowClipPlanes.iter().all(|plane| {
            dot4(*plane, x1, y1, z1) > 0.0
                || dot4(*plane, x2, y1, z1) > 0.0
                || dot4(*plane, x1, y2, z1) > 0.0
                || dot4(*plane, x2, y2, z1) > 0.0
                || dot4(*plane, x1, y1, z2) > 0.0
                || dot4(*plane, x2, y1, z2) > 0.0
                || dot4(*plane, x1, y2, z2) > 0.0
                || dot4(*plane, x2, y2, z2) > 0.0
        })
    }

    pub fn planeCount(&self) -> usize {
        self.shadowClipPlanes.len()
    }
}

fn adjacentPlaneIndices(index: usize) -> [usize; 4] {
    match index {
        0 | 1 => [2, 3, 4, 5],
        2 | 3 => [0, 1, 4, 5],
        4 | 5 => [0, 1, 2, 3],
        _ => unreachable!("OptiFine camera frustum has exactly six planes"),
    }
}

fn makeShadowPlane(positivePlane: [f32; 4], negativePlane: [f32; 4], sun: [f32; 4]) -> [f32; 4] {
    let intersection = cross3(positivePlane, negativePlane);
    let mut shadowPlane = normalize3(cross3(intersection, sun));
    let dotPlanes = dot3(positivePlane, negativePlane);
    let dotShadowNegative = dot3(shadowPlane, negativePlane);
    let distanceShadowNegative = distance3(shadowPlane, scale3(negativePlane, dotShadowNegative));
    let distancePositiveNegative = distance3(positivePlane, scale3(negativePlane, dotPlanes));
    let positiveFactor = distanceShadowNegative / distancePositiveNegative;

    let dotShadowPositive = dot3(shadowPlane, positivePlane);
    let distanceShadowPositive = distance3(shadowPlane, scale3(positivePlane, dotShadowPositive));
    let distanceNegativePositive = distance3(negativePlane, scale3(positivePlane, dotPlanes));
    let negativeFactor = distanceShadowPositive / distanceNegativePositive;
    shadowPlane[3] = positivePlane[3] * positiveFactor + negativePlane[3] * negativeFactor;
    shadowPlane
}

fn assignPlane(a: f32, b: f32, c: f32, d: f32) -> [f32; 4] {
    let length = (a * a + b * b + c * c).sqrt();
    [a / length, b / length, c / length, d / length]
}

fn normalize3(mut value: [f32; 4]) -> [f32; 4] {
    let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt();
    let divisor = if length == 0.0 { 1.0 } else { length };
    value[0] /= divisor;
    value[1] /= divisor;
    value[2] /= divisor;
    value
}

fn cross3(left: [f32; 4], right: [f32; 4]) -> [f32; 4] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
        0.0,
    ]
}

fn dot3(left: [f32; 4], right: [f32; 4]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn dot4(plane: [f32; 4], x: f64, y: f64, z: f64) -> f64 {
    plane[0] as f64 * x + plane[1] as f64 * y + plane[2] as f64 * z + plane[3] as f64
}

fn scale3(value: [f32; 4], scalar: f32) -> [f32; 4] {
    [
        value[0] * scalar,
        value[1] * scalar,
        value[2] * scalar,
        value[3],
    ]
}

fn distance3(left: [f32; 4], right: [f32; 4]) -> f32 {
    let x = left[0] - right[0];
    let y = left[1] - right[1];
    let z = left[2] - right[2];
    (x * x + y * y + z * z).sqrt()
}

fn multiply4(left: [f32; 16], right: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4)
                .map(|index| left[index * 4 + row] * right[column * 4 + index])
                .sum();
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity4() -> [f32; 16] {
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    }

    #[test]
    fn builds_optifine_shadow_silhouette_planes() {
        let helper =
            ClippingHelperShadow::fromMatrices(identity4(), identity4(), [0.0, 1.0, 0.0, 0.0]);
        assert!(helper.planeCount() > 0);
        assert!(helper.planeCount() <= 10);
        assert!(helper.isBoxInFrustum(-0.25, -0.25, -0.25, 0.25, 0.25, 0.25, [0.0, 0.0, 0.0],));
    }
}
