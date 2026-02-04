use num_bigint::BigUint;
use num_traits::{Num, Zero};

pub struct PasswordEncryptor {
    e: BigUint,
    n: BigUint,
    chunk_size: usize,
}

impl PasswordEncryptor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        const PUBLIC_EXPONENT: &str = "10001";
        const PUBLIC_MODULUS: &str = 
            "94dd2a8675fb779e6b9f7103698634cd400f27a154afa67af6166a43fc26417222a79506d34cacc7641946abda1785b7acf9910ad6a0978c91ec84d40b71d2891379af19ffb333e7517e390bd26ac312fe940c340466b4a5d4af1d65c3b5944078f96a1a51a5a53e4bc302818b7c9f63c4a1b07bd7d874cef1c3d4b2f5eb7871";

        let e = BigUint::from_str_radix(PUBLIC_EXPONENT, 16)?;
        let n = BigUint::from_str_radix(PUBLIC_MODULUS, 16)?;
        
        // 计算模数的位数
        let n_bits = n.bits() as usize;
        // chunk_size = 2 * (n的十六进制位数 - 1)
        // 每个十六进制字符 = 4 bits, 每2个字节 = 1个digit
        let chunk_size = 2 * ((n_bits + 7) / 8 - 1);

        Ok(Self { e, n, chunk_size })
    }

    /// 完全匹配JavaScript的encryptedPassword实现
    pub fn encrypt_password(&self, password: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 1. 反转密码字符串
        let reversed: String = password.chars().rev().collect();
        
        // 2. 转换为字节数组
        let mut bytes: Vec<u8> = reversed.bytes().collect();
        
        // 3. 零填充到chunk_size的倍数
        while bytes.len() % self.chunk_size != 0 {
            bytes.push(0);
        }
        
        // 4. 分块加密
        let mut result_parts = Vec::new();
        
        for chunk in bytes.chunks(self.chunk_size) {
            // 构建block（模拟JavaScript的BigInt构建方式）
            let mut block = BigUint::zero();
            
            // 每次处理2个字节，组成一个16位的digit
            for (j, pair) in chunk.chunks(2).enumerate() {
                let byte0 = pair[0] as u64;
                let byte1 = if pair.len() > 1 { pair[1] as u64 } else { 0 };
                
                // 组合成16位数字: byte0 + (byte1 << 8)
                let digit = byte0 | (byte1 << 8);
                
                // 添加到block: block += digit * (2^16)^j
                block += BigUint::from(digit) << (j * 16);
            }
            
            // 模幂运算: crypt = block^e mod n
            let encrypted_block = block.modpow(&self.e, &self.n);
            
            // 转换为十六进制字符串
            let hex_string = format!("{:x}", encrypted_block);
            result_parts.push(hex_string);
        }
        
        // 5. 用空格连接各部分
        Ok(result_parts.join(" "))
    }
}

impl Default for PasswordEncryptor {
    fn default() -> Self {
        Self::new().expect("Failed to create password encryptor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_password() {
        let encryptor = PasswordEncryptor::new().unwrap();

        // 测试1: 模拟密码+MAC格式
        let password1 = "testPassword123>aabbccddeeff";
        let encrypted1 = encryptor.encrypt_password(password1).unwrap();

        // 测试2: 更长的密码
        let password2 = "anotherTestPassword456!";
        let encrypted2 = encryptor.encrypt_password(password2).unwrap();

        // 验证输出格式
        assert!(!encrypted1.is_empty());
        assert!(!encrypted2.is_empty());

        // 验证确定性：相同输入应产生相同输出
        let encrypted1_again = encryptor.encrypt_password(password1).unwrap();
        assert_eq!(encrypted1, encrypted1_again, "加密应该是确定性的");
    }
}