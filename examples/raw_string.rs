fn main() {
    let var1 = "test1";

    let json = format!(
        r#"{{
    "type": "type1", 
    "type2": {}
}}"#,
        var1
    );
    let s = format!("{} some string", var1);
    println!("{}", json);
    println!("{}", s);
    // let s = r#"{}"#;
    println!(r#"{}"#, "random");
    let s1 = format!("{{}}");
    println!("{}", s1);

    let foo = r##"ra\nd\om"##;
    println!("{:?}", foo); // 这里是 debug print

    let foo = r#"ra\nd\om"#;
    println!("{}", foo); 

    let bar = r##"foo #"# bar"##;
    println!("{}", bar);
}
