use std::rc::Rc;

use rquickjs::class::{Class, ClassId, JsClass, Readable, Trace, Tracer};
use rquickjs::function::{Constructor, This};
use rquickjs::{Context, Ctx, FromJs, Function, IntoJs, Object, Result, Runtime, Value};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RustData(Rc<str>);

impl<'js> Trace<'js> for RustData {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> FromJs<'js> for RustData {
    fn from_js(_: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Ok(Class::<Self>::from_value(&value)?.try_borrow()?.clone())
    }
}

impl<'js> IntoJs<'js> for RustData {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Class::instance(ctx.clone(), self).into_js(ctx)
    }
}

impl<'js> JsClass<'js> for RustData {
    const NAME: &'static str = "RustData";

    type Mutable = Readable;

    fn class_id() -> &'static ClassId {
        static ID: ClassId = ClassId::new();
        &ID
    }

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;

        let to_string = Function::new(ctx.clone(), |this: This<Self>| this.0 .0.to_string())?
            .with_name("toString")?;
        proto.prop("toString", to_string)?;

        let lt = Function::new(ctx.clone(), |this: This<Self>, other: Self| this.0 < other)?
            .with_name("lt")?;
        proto.prop("lt", lt)?;

        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        Ok(Some(Constructor::new_class::<Self, _, _>(
            ctx.clone(),
            |s: String| RustData(s.into()),
        )?))
    }
}

pub fn sort_userdata(run: impl FnOnce(&mut dyn FnMut())) -> Result<()> {
    let rt = Runtime::new()?;
    let context = Context::full(&rt)?;

    context.with(|ctx| {
        let globals = ctx.globals();

        // let print = Function::new(ctx.clone(), |s: String| println!("{s}"))?.with_name("print")?;
        // globals.set("print", print)?;

        let rand =
            Function::new(ctx.clone(), |n: u32| rand::random::<u32>() % n)?.with_name("rand")?;
        globals.set("rand", rand).unwrap();

        Class::<RustData>::define(&globals)?;

        ctx.eval::<(), _>(include_str!("../scripts/sort_userdata.js"))?;

        let func = globals.get::<_, Function>("bench")?;
        run(&mut || func.call::<_, ()>(()).unwrap());

        Ok(())
    })
}
