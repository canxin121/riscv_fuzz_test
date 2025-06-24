pub fn get_register_name(reg_num: usize) -> &'static str {
    match reg_num {
        0 => "zero",
        1 => "ra",
        2 => "sp",
        3 => "gp",
        4 => "tp",
        5 => "t0",
        6 => "t1",
        7 => "t2",
        8 => "s0",
        9 => "s1",
        10 => "a0",
        11 => "a1",
        12 => "a2",
        13 => "a3",
        14 => "a4",
        15 => "a5",
        16 => "a6",
        17 => "a7",
        18 => "s2",
        19 => "s3",
        20 => "s4",
        21 => "s5",
        22 => "s6",
        23 => "s7",
        24 => "s8",
        25 => "s9",
        26 => "s10",
        27 => "s11",
        28 => "t3",
        29 => "t4",
        30 => "t5",
        31 => "t6",
        _ => "invalid",
    }
}

/// 获取异常描述
pub fn get_exception_description(mcause: u64) -> String {
    let interrupt = (mcause >> 63) & 1 == 1;
    let exception_code = mcause & 0x7FFFFFFFFFFFFFFF;

    if interrupt {
        match exception_code {
            0 => "用户软件中断".to_string(),
            1 => "监管者软件中断".to_string(),
            3 => "机器软件中断".to_string(),
            4 => "用户定时器中断".to_string(),
            5 => "监管者定时器中断".to_string(),
            7 => "机器定时器中断".to_string(),
            8 => "用户外部中断".to_string(),
            9 => "监管者外部中断".to_string(),
            11 => "机器外部中断".to_string(),
            _ => format!("未知中断 (代码={})", exception_code),
        }
    } else {
        match exception_code {
            0 => "指令地址未对齐".to_string(),
            1 => "指令访问故障".to_string(),
            2 => "非法指令".to_string(),
            3 => "断点".to_string(),
            4 => "加载地址未对齐".to_string(),
            5 => "加载访问故障".to_string(),
            6 => "存储/原子地址未对齐".to_string(),
            7 => "存储/原子访问故障".to_string(),
            8 => "来自用户模式的环境调用".to_string(),
            9 => "来自监管者模式的环境调用".to_string(),
            11 => "来自机器模式的环境调用".to_string(),
            12 => "指令页面故障".to_string(),
            13 => "加载页面故障".to_string(),
            15 => "存储/原子页面故障".to_string(),
            _ => format!("未知异常 (代码={})", exception_code),
        }
    }
}

/// 获取寄存器的详细描述
pub fn get_register_description(reg_num: usize) -> &'static str {
    match reg_num {
        0 => "零寄存器 (总是为0)",
        1 => "返回地址寄存器",
        2 => "栈指针寄存器",
        3 => "全局指针寄存器",
        4 => "线程指针寄存器",
        5..=7 => "临时寄存器",
        8 => "帧指针/保存寄存器",
        9 => "保存寄存器",
        10..=11 => "函数参数/返回值寄存器",
        12..=17 => "函数参数寄存器",
        18..=27 => "保存寄存器",
        28..=31 => "临时寄存器",
        _ => "无效寄存器",
    }
}

/// 获取浮点寄存器的ABI名称
pub fn get_float_register_name(reg_num: usize) -> String {
    match reg_num {
        0..=7 => format!("ft{}", reg_num),
        8..=9 => format!("fs{}", reg_num - 8),
        10..=17 => format!("fa{}", reg_num - 10),
        18..=27 => format!("fs{}", reg_num - 18 + 2),
        28..=31 => format!("ft{}", reg_num - 28 + 8),
        _ => "invalid".to_string(),
    }
}

/// 获取浮点寄存器的详细描述
pub fn get_float_register_description(reg_num: usize) -> &'static str {
    match reg_num {
        0..=7 => "临时浮点寄存器",
        8..=9 => "保存浮点寄存器",
        10..=17 => "浮点参数/返回值寄存器",
        18..=27 => "保存浮点寄存器",
        28..=31 => "临时浮点寄存器",
        _ => "无效浮点寄存器",
    }
}

/// 格式化十六进制值为易读格式
pub fn format_hex_value(value: u64) -> String {
    format!("0x{:016X}", value)
}

/// 获取CSR寄存器的详细描述
pub fn get_csr_description(csr_name: &str) -> &'static str {
    match csr_name {
        "mstatus" => "机器状态寄存器 - 控制全局中断使能、特权级别等",
        "misa" => "机器ISA寄存器 - 指示支持的指令集扩展",
        "medeleg" => "机器异常委托寄存器 - 控制异常的委托",
        "mideleg" => "机器中断委托寄存器 - 控制中断的委托",
        "mie" => "机器中断使能寄存器 - 控制各种中断的使能",
        "mtvec" => "机器陷阱向量基地址寄存器 - 异常和中断的处理地址",
        "mcounteren" => "机器计数器使能寄存器 - 控制性能计数器的访问",
        "mscratch" => "机器临时寄存器 - 用于保存临时数据",
        "mepc" => "机器异常程序计数器 - 保存异常发生时的PC",
        "mcause" => "机器陷阱原因寄存器 - 指示异常或中断的原因",
        "mtval" => "机器陷阱值寄存器 - 保存异常相关的地址或指令",
        "mip" => "机器中断挂起寄存器 - 显示当前挂起的中断",
        "mcycle" => "机器周期计数器 - 记录CPU周期数",
        "minstret" => "机器指令退役计数器 - 记录已执行的指令数",
        "mvendorid" => "机器厂商ID寄存器 - 标识硬件厂商",
        "marchid" => "机器架构ID寄存器 - 标识微架构",
        "mimpid" => "机器实现ID寄存器 - 标识具体实现版本",
        "mhartid" => "机器硬件线程ID寄存器 - 标识当前hart",
        "fcsr" => "浮点控制和状态寄存器 - 控制浮点运算模式和异常",
        _ => "未知CSR寄存器",
    }
}

/// 生成当前时间戳
pub fn get_current_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
