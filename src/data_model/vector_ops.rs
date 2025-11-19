#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub mod unsafe_ops {
    use super::*;

    #[cfg(target_arch = "x86_64")]
    pub fn sum_f32_fast(data: &[f32]) -> f32 {
        if data.is_empty() {
            return 0.0;
        }

        #[cfg(target_feature = "avx2")]
        {
            if is_x86_feature_detected!("avx2") {
                return unsafe { sum_f32_avx2(data) };
            }
        }

        sum_f32_scalar(data)
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn sum_f32_fast(data: &[f32]) -> f32 {
        sum_f32_scalar(data)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn sum_f32_avx2(data: &[f32]) -> f32 {
        let chunks = data.chunks_exact(8);
        let remainder = chunks.remainder();
        let mut sum_vec = _mm256_setzero_ps();

        for chunk in chunks {
            let values = _mm256_loadu_ps(chunk.as_ptr());
            sum_vec = _mm256_add_ps(sum_vec, values);
        }

        let mut result = [0.0f32; 8];
        _mm256_storeu_ps(result.as_mut_ptr(), sum_vec);
        let sum = result.iter().sum::<f32>();

        sum + remainder.iter().sum::<f32>()
    }

    fn sum_f32_scalar(data: &[f32]) -> f32 {
        unsafe {
            let ptr = data.as_ptr();
            let len = data.len();
            let mut sum = 0.0f32;

            for i in 0..len {
                sum += *ptr.add(i);
            }

            sum
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn sum_sq_diff_f32_fast(data: &[f32], mean: f32) -> f32 {
        if data.is_empty() {
            return 0.0;
        }

        #[cfg(target_feature = "avx2")]
        {
            if is_x86_feature_detected!("avx2") {
                return unsafe { sum_sq_diff_f32_avx2(data, mean) };
            }
        }

        sum_sq_diff_f32_scalar(data, mean)
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn sum_sq_diff_f32_fast(data: &[f32], mean: f32) -> f32 {
        sum_sq_diff_f32_scalar(data, mean)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn sum_sq_diff_f32_avx2(data: &[f32], mean: f32) -> f32 {
        let mean_vec = _mm256_set1_ps(mean);
        let chunks = data.chunks_exact(8);
        let remainder = chunks.remainder();
        let mut sum_sq = _mm256_setzero_ps();

        for chunk in chunks {
            let values = _mm256_loadu_ps(chunk.as_ptr());
            let diff = _mm256_sub_ps(values, mean_vec);
            let sq = _mm256_mul_ps(diff, diff);
            sum_sq = _mm256_add_ps(sum_sq, sq);
        }

        let mut result = [0.0f32; 8];
        _mm256_storeu_ps(result.as_mut_ptr(), sum_sq);
        let sum = result.iter().sum::<f32>();

        let remainder_sum = remainder.iter().map(|&x| {
            let diff = x - mean;
            diff * diff
        }).sum::<f32>();

        sum + remainder_sum
    }

    fn sum_sq_diff_f32_scalar(data: &[f32], mean: f32) -> f32 {
        unsafe {
            let ptr = data.as_ptr();
            let len = data.len();
            let mut sum_sq = 0.0f32;

            for i in 0..len {
                let diff = *ptr.add(i) - mean;
                sum_sq += diff * diff;
            }

            sum_sq
        }
    }

    pub fn mean_f32_fast(data: &[f32]) -> Option<f32> {
        if data.is_empty() {
            return None;
        }
        Some(sum_f32_fast(data) / data.len() as f32)
    }

    pub fn variance_f32_fast(data: &[f32]) -> Option<f32> {
        if data.is_empty() {
            return None;
        }
        let mean = mean_f32_fast(data)?;
        let sum_sq_diff = sum_sq_diff_f32_fast(data, mean);
        Some(sum_sq_diff / data.len() as f32)
    }

    pub fn std_dev_f32_fast(data: &[f32]) -> Option<f32> {
        variance_f32_fast(data).map(|v| v.sqrt())
    }

    pub fn copy_f32_fast(src: &[f32], dst: &mut [f32]) {
        let len = src.len().min(dst.len());
        if len == 0 {
            return;
        }

        unsafe {
            std::ptr::copy_nonoverlapping(
                src.as_ptr(),
                dst.as_mut_ptr(),
                len,
            );
        }
    }

    pub fn add_vectors_f32_fast(a: &[f32], b: &[f32], result: &mut [f32]) {
        let len = a.len().min(b.len()).min(result.len());
        if len == 0 {
            return;
        }

        unsafe {
            let a_ptr = a.as_ptr();
            let b_ptr = b.as_ptr();
            let result_ptr = result.as_mut_ptr();

            for i in 0..len {
                *result_ptr.add(i) = *a_ptr.add(i) + *b_ptr.add(i);
            }
        }
    }

    pub fn subtract_vectors_f32_fast(a: &[f32], b: &[f32], result: &mut [f32]) {
        let len = a.len().min(b.len()).min(result.len());
        if len == 0 {
            return;
        }

        unsafe {
            let a_ptr = a.as_ptr();
            let b_ptr = b.as_ptr();
            let result_ptr = result.as_mut_ptr();

            for i in 0..len {
                *result_ptr.add(i) = *a_ptr.add(i) - *b_ptr.add(i);
            }
        }
    }

    pub fn multiply_vectors_f32_fast(a: &[f32], b: &[f32], result: &mut [f32]) {
        let len = a.len().min(b.len()).min(result.len());
        if len == 0 {
            return;
        }

        unsafe {
            let a_ptr = a.as_ptr();
            let b_ptr = b.as_ptr();
            let result_ptr = result.as_mut_ptr();

            for i in 0..len {
                *result_ptr.add(i) = *a_ptr.add(i) * *b_ptr.add(i);
            }
        }
    }

    pub fn scale_vector_f32_fast(data: &[f32], factor: f32, result: &mut [f32]) {
        let len = data.len().min(result.len());
        if len == 0 {
            return;
        }

        unsafe {
            let data_ptr = data.as_ptr();
            let result_ptr = result.as_mut_ptr();

            for i in 0..len {
                *result_ptr.add(i) = *data_ptr.add(i) * factor;
            }
        }
    }

    pub fn max_f32_fast(data: &[f32]) -> Option<f32> {
        if data.is_empty() {
            return None;
        }

        unsafe {
            let ptr = data.as_ptr();
            let len = data.len();
            let mut max = *ptr;

            for i in 1..len {
                let val = *ptr.add(i);
                if val > max {
                    max = val;
                }
            }

            Some(max)
        }
    }

    pub fn min_f32_fast(data: &[f32]) -> Option<f32> {
        if data.is_empty() {
            return None;
        }

        unsafe {
            let ptr = data.as_ptr();
            let len = data.len();
            let mut min = *ptr;

            for i in 1..len {
                let val = *ptr.add(i);
                if val < min {
                    min = val;
                }
            }

            Some(min)
        }
    }

    pub fn dot_product_f32_fast(a: &[f32], b: &[f32]) -> f32 {
        let len = a.len().min(b.len());
        if len == 0 {
            return 0.0;
        }

        unsafe {
            let a_ptr = a.as_ptr();
            let b_ptr = b.as_ptr();
            let mut sum = 0.0f32;

            for i in 0..len {
                sum += *a_ptr.add(i) * *b_ptr.add(i);
            }

            sum
        }
    }
}

#[cfg(test)]
mod tests {
    use super::unsafe_ops::*;

    #[test]
    fn test_sum_f32_fast() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(sum_f32_fast(&data), 15.0);
    }

    #[test]
    fn test_mean_f32_fast() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(mean_f32_fast(&data), Some(3.0));
    }

    #[test]
    fn test_variance_f32_fast() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let variance = variance_f32_fast(&data).unwrap();
        assert!((variance - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_max_min_f32_fast() {
        let data = vec![1.0, 5.0, 3.0, 2.0, 4.0];
        assert_eq!(max_f32_fast(&data), Some(5.0));
        assert_eq!(min_f32_fast(&data), Some(1.0));
    }
}

