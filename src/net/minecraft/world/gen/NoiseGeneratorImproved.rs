use crate::compat::Java::JavaRandom;
use crate::net::minecraft::world::gen::NoiseGenerator::NoiseGenerator;

const GRAD_X: [f64; 16] = [1.0,-1.0,1.0,-1.0,1.0,-1.0,1.0,-1.0,0.0,0.0,0.0,0.0,1.0,0.0,-1.0,0.0];
const GRAD_Y: [f64; 16] = [1.0,1.0,-1.0,-1.0,0.0,0.0,0.0,0.0,1.0,-1.0,1.0,-1.0,1.0,-1.0,1.0,-1.0];
const GRAD_Z: [f64; 16] = [0.0,0.0,0.0,0.0,1.0,1.0,-1.0,-1.0,1.0,1.0,-1.0,-1.0,0.0,1.0,0.0,-1.0];
const GRAD_2X: [f64; 16] = [1.0,-1.0,1.0,-1.0,1.0,-1.0,1.0,-1.0,0.0,0.0,0.0,0.0,1.0,0.0,-1.0,0.0];
const GRAD_2Z: [f64; 16] = [0.0,0.0,0.0,0.0,1.0,1.0,-1.0,-1.0,1.0,1.0,-1.0,-1.0,0.0,1.0,0.0,-1.0];

/// Exact Rust port of MCP 1.12.2 `NoiseGeneratorImproved`.
#[derive(Debug, Clone)]
pub struct NoiseGeneratorImproved {
    permutations: [i32; 512],
    pub xCoord: f64,
    pub yCoord: f64,
    pub zCoord: f64,
}

impl NoiseGenerator for NoiseGeneratorImproved {}

impl NoiseGeneratorImproved {
    pub fn new(random: &mut JavaRandom) -> Self {
        let mut permutations=[0_i32;512];
        let xCoord=random.next_f64()*256.0;
        let yCoord=random.next_f64()*256.0;
        let zCoord=random.next_f64()*256.0;
        for i in 0..256 { permutations[i]=i as i32; }
        for l in 0..256 {
            let j=random.next_i32_bound((256-l) as i32) as usize+l;
            permutations.swap(l,j);
            permutations[l+256]=permutations[l];
        }
        Self{permutations,xCoord,yCoord,zCoord}
    }

    #[inline]
    pub fn lerp(&self, amount:f64, start:f64, end:f64)->f64 { start+amount*(end-start) }

    #[inline]
    pub fn grad2(&self, hash:i32, x:f64, z:f64)->f64 {
        let i=(hash&15) as usize;
        GRAD_2X[i]*x+GRAD_2Z[i]*z
    }

    #[inline]
    pub fn grad(&self, hash:i32, x:f64, y:f64, z:f64)->f64 {
        let i=(hash&15) as usize;
        GRAD_X[i]*x+GRAD_Y[i]*y+GRAD_Z[i]*z
    }

    /// MCP `populateNoiseArray`; the destination is accumulated into rather
    /// than overwritten, because `NoiseGeneratorOctaves` combines octaves.
    #[allow(clippy::too_many_arguments)]
    pub fn populateNoiseArray(&self, noiseArray:&mut [f64], xOffset:f64, yOffset:f64, zOffset:f64,
        xSize:i32, ySize:i32, zSize:i32, xScale:f64, yScale:f64, zScale:f64, noiseScale:f64) {
        let expected=(xSize as usize).saturating_mul(ySize as usize).saturating_mul(zSize as usize);
        assert!(noiseArray.len()>=expected,"noiseArray must hold xSize*ySize*zSize entries");
        if ySize==1 {
            let mut l5=0usize;
            let d16=1.0/noiseScale;
            for j2 in 0..xSize {
                let mut d17=xOffset+j2 as f64*xScale+self.xCoord;
                let mut i6=d17 as i32;
                if d17<i6 as f64 { i6-=1; }
                let k2=(i6&255) as usize;
                d17-=i6 as f64;
                let d18=d17*d17*d17*(d17*(d17*6.0-15.0)+10.0);
                for j6 in 0..zSize {
                    let mut d19=zOffset+j6 as f64*zScale+self.zCoord;
                    let mut k6=d19 as i32;
                    if d19<k6 as f64 { k6-=1; }
                    let l6=(k6&255) as usize;
                    d19-=k6 as f64;
                    let d20=d19*d19*d19*(d19*(d19*6.0-15.0)+10.0);
                    let i5=self.permutations[k2] as usize;
                    let j5=(self.permutations[i5] as usize)+l6;
                    let j=self.permutations[k2+1] as usize;
                    let k5=(self.permutations[j] as usize)+l6;
                    let d14=self.lerp(d18,
                        self.grad2(self.permutations[j5],d17,d19),
                        self.grad(self.permutations[k5],d17-1.0,0.0,d19));
                    let d15=self.lerp(d18,
                        self.grad(self.permutations[j5+1],d17,0.0,d19-1.0),
                        self.grad(self.permutations[k5+1],d17-1.0,0.0,d19-1.0));
                    let d21=self.lerp(d20,d14,d15);
                    noiseArray[l5]+=d21*d16;
                    l5+=1;
                }
            }
        } else {
            let d0=1.0/noiseScale;
            let mut index=0usize;
            let mut previous_l4=-1_i32;
            let (mut i1,mut j1,mut l1,mut i2)=(0usize,0usize,0usize,0usize);
            let (mut d1,mut d2,mut d3,mut d4)=(0.0,0.0,0.0,0.0);
            for l2 in 0..xSize {
                let mut d5=xOffset+l2 as f64*xScale+self.xCoord;
                let mut i3=d5 as i32;
                if d5<i3 as f64 { i3-=1; }
                let j3=(i3&255) as usize;
                d5-=i3 as f64;
                let d6=d5*d5*d5*(d5*(d5*6.0-15.0)+10.0);
                for k3 in 0..zSize {
                    let mut d7=zOffset+k3 as f64*zScale+self.zCoord;
                    let mut l3=d7 as i32;
                    if d7<l3 as f64 { l3-=1; }
                    let i4=(l3&255) as usize;
                    d7-=l3 as f64;
                    let d8=d7*d7*d7*(d7*(d7*6.0-15.0)+10.0);
                    for j4 in 0..ySize {
                        let mut d9=yOffset+j4 as f64*yScale+self.yCoord;
                        let mut k4=d9 as i32;
                        if d9<k4 as f64 { k4-=1; }
                        let l4=k4&255;
                        d9-=k4 as f64;
                        let d10=d9*d9*d9*(d9*(d9*6.0-15.0)+10.0);
                        if j4==0 || l4!=previous_l4 {
                            previous_l4=l4;
                            let l=(self.permutations[j3] as usize)+(l4 as usize);
                            i1=(self.permutations[l] as usize)+i4;
                            j1=(self.permutations[l+1] as usize)+i4;
                            let k1=(self.permutations[j3+1] as usize)+(l4 as usize);
                            l1=(self.permutations[k1] as usize)+i4;
                            i2=(self.permutations[k1+1] as usize)+i4;
                            d1=self.lerp(d6,self.grad(self.permutations[i1],d5,d9,d7),self.grad(self.permutations[l1],d5-1.0,d9,d7));
                            d2=self.lerp(d6,self.grad(self.permutations[j1],d5,d9-1.0,d7),self.grad(self.permutations[i2],d5-1.0,d9-1.0,d7));
                            d3=self.lerp(d6,self.grad(self.permutations[i1+1],d5,d9,d7-1.0),self.grad(self.permutations[l1+1],d5-1.0,d9,d7-1.0));
                            d4=self.lerp(d6,self.grad(self.permutations[j1+1],d5,d9-1.0,d7-1.0),self.grad(self.permutations[i2+1],d5-1.0,d9-1.0,d7-1.0));
                        }
                        let d11=self.lerp(d10,d1,d2);
                        let d12=self.lerp(d10,d3,d4);
                        let d13=self.lerp(d8,d11,d12);
                        noiseArray[index]+=d13*d0;
                        index+=1;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_coordinates_match_java_random_sequence() {
        let mut random=JavaRandom::new(12345);
        let noise=NoiseGeneratorImproved::new(&mut random);
        assert_eq!(noise.xCoord.to_bits(),(92.62159543308078_f64).to_bits());
        assert_eq!(noise.yCoord.to_bits(),(238.8463322338665_f64).to_bits());
        assert_eq!(noise.zCoord.to_bits(),(213.27138533658206_f64).to_bits());
    }

    #[test]
    fn single_y_branch_matches_mcp_java_golden_values(){
        let mut random=JavaRandom::new(98765);
        let noise=NoiseGeneratorImproved::new(&mut random);
        let mut values=[0.0_f64;6];
        noise.populateNoiseArray(&mut values,-2.25,0.0,3.75,2,1,3,0.125,1.0,0.25,0.5);
        let expected=[-0.28910070419203715,-0.2424861044474541,-0.42796135428714965,-0.6724937814307574,-0.5240347723996917,-0.4926732505002159];
        for (actual,expected) in values.iter().zip(expected){assert!((actual-expected).abs()<1.0e-14,"{actual} != {expected}");}
    }
}
