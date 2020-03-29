use rlua::Lua;
use rlua::Result;
use rlua::Function;

fn main() -> Result<()> {
    println!(r" ___         _          ___          __ ");
    println!(r"| _ \_  _ __| |_ _  _  | _ \___ ___ / _|");
    println!(r"|   / || (_-<  _| || | |   / -_) -_)  _|");
    println!(r"|_|_\\_,_/__/\__|\_, | |_|_\___\___|_|  ");
    println!(r"                 |__/                   "); 


    let lua = Lua::new();

    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();

        let inputs = lua_ctx.create_table()?;
        inputs.set("Tank_Temperature", 75.9)?;
        globals.set("inputs", inputs)?;

        let outputs = lua_ctx.create_table()?;
        outputs.set("Heater_Outlet", 2)?;
        globals.set("outputs", outputs)?;

        lua_ctx.load(
            r#"
            function protect(tbl)
                return setmetatable({}, {
                    __index = tbl,
                    __newindex = function(t, key, value)
                        error("attempting to change constant " ..
                            tostring(key) .. " to " .. tostring(value), 2)
                    end
                })
            end
            inputs = protect(inputs)
            outputs = protect(outputs)
            "#
        ).exec()?;

        let blah = lua_ctx
            .load(
                r#"
                    if inputs.Tank_Temperature < 78 then
                        return 1
                    elseif inputs.Tank_Temperature > 80 then
                        return 0
                    else
                        return outputs.Heater_Outlet
                    end
            "#,
            )
            .eval::<i32>()?;
        
        
        print!("*** {:?}", blah);

        Ok(())
    })?;

    Ok(())
}
