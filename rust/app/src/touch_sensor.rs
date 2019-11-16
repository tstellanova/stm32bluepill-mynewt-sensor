use embedded_hal::{
    self,
    blocking::delay::DelayMs,
    digital::v2::OutputPin,
};
use mynewt::{
    result::*,
    hw::hal,
    kernel::os,
    sys::console,
};
use crate::mynewt_hal::{
    MynewtDelay,
    MynewtGPIO,
};

/// Probe the I2C bus
pub fn probe() -> MynewtResult<()> {
    for addr in 1..127 {
        let rc = unsafe { hal::hal_i2c_master_probe(1, addr, 1000) };
        if rc != hal::HAL_I2C_ERR_ADDR_NACK as i32 {
            //  I2C device found
            console::print("0x"); console::printhex(addr); console::print(": ");
            console::printhex(rc as u8); console::print("\n"); console::flush();
        }
    }
    console::print("Done\n"); console::flush();
    Ok(())
}

/* I2C devices found:
0x18: 00
0x44: 00 */

pub fn test() -> MynewtResult<()> {
    let mut delay = MynewtDelay{};
    let mut reset = MynewtGPIO::new(10);  //  P0.10/NFC2: TP_RESET

    reset.set_low() ? ;
    //reset.set_high() ? ;

    delay.delay_ms(20);

    reset.set_high() ? ;
    //reset.set_low() ? ;

    delay.delay_ms(200);
    delay.delay_ms(200);

    let addr = 0x15;  //  From reference code

    //  let addr = 0x18;  //  From probe
    //  let addr = 0x44;  //  From probe

    //  Register to be read
    //  let register = 0x00;
    let register = 0xA3;  //  HYN_REG_CHIP_ID
    //  let register = 0x8F;  //  HYN_REG_INT_CNT

    for _ in 1..20 {
        read_register(addr, register) ? ;
        delay.delay_ms(200);
    }
    Ok(())
}

fn read_register(addr: u8, register: u8) -> MynewtResult<()> {
    //  first the register address must be sent in write mode (slave address xxxxxxx0). 
    unsafe { 
        I2C_BUFFER[0] = register;
        I2C_DATA.address = addr; // (addr << 1) + 0;
        I2C_DATA.len = I2C_BUFFER.len() as u16;
        I2C_DATA.buffer = I2C_BUFFER.as_mut_ptr();
    };
    //  Then either a stop or a repeated start condition must be generated. 
    let rc1 = unsafe { hal::hal_i2c_master_write(1, &mut I2C_DATA, 1000, 0) };

    //  After this the slave is addressed in read mode (RW = ‘1’) at address xxxxxxx1, 
    unsafe { 
        I2C_BUFFER[0] = 0x00;
        I2C_DATA.address = addr; // (addr << 1) + 1;
        I2C_DATA.len = I2C_BUFFER.len() as u16;
        I2C_DATA.buffer = I2C_BUFFER.as_mut_ptr();
    };
    //  after which the slave sends out data from auto-incremented register addresses until a NOACKM and stop condition occurs.
    let rc2 = unsafe { hal::hal_i2c_master_read(1, &mut I2C_DATA, 1000, 1) };

    console::print("write 0x"); console::printhex(addr); console::print(", rc: ");
    console::printhex(rc1 as u8); console::print("\n"); console::flush();

    console::print("read 0x"); console::printhex(addr); console::print(", rc: ");
    console::printhex(rc2 as u8); console::print("\n"); console::flush();

    console::print("read value 0x"); console::printhex(unsafe { I2C_BUFFER[0] }); 
    console::print("\n"); console::flush();
    console::print("Done\n"); console::flush();
    Ok(())
}

static mut I2C_BUFFER: [u8; 1] =  [ 0 ];

static mut I2C_DATA: hal::hal_i2c_master_data = hal::hal_i2c_master_data {
    address: 0,
    len:     0,
    buffer:  core::ptr::null_mut(),
};

/*
write 0x18: 00
read 0x18: 00
value 0x00
write 0x44: 00
read 0x44: 00
value 0xa3
write 0x98: 00
read 0x98: 00
value 0x00
write 0xc4: 00
read 0xc4: 00
value 0xa3
*/

/*
From http://www.hynitron.com/upload/1408075960.pdf:
//初始化设置
I2Cm_Write_Data(0xAF, 0x3A); //打开写保护
I2Cm_Write_Data(0x1F,0x00); //关闭扫描
I2Cm_Write_Data(0x12, 0x18); //设置允许多个button被同时触发，调试时关闭噪声保护

//IRQ处理
// 读取所有Button的状态
Button=I2Cm_Read_Data(0x22); //读取感应通道0-7的状态寄存器用于判断其触摸状态
I2Cm_Write_Data(0x1F, 0x01); //清除IRQ（不打开写保护时仅能清除IRQ位，其它控制位不变）
// 客户可以根据Button的信息读取相关感应通道的Signal值并对数据进行进一步处理
*/