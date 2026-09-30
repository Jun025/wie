use alloc::{boxed::Box, format};

use jvm::{ClassInstance, Field, JavaValue, Jvm, Result as JvmResult};

// A native class's own code means its own field. jvm.get_field/put_field resolve the name from the
// instance's runtime class, so a game subclass that declares a field of the same name and type
// captures the read — 34ab350dc98a's Card subclass returned its own x = 240, w = 0 and painted off
// screen. These resolve the field from `class_name`, the class that declared it. Use them for a
// field a game subclass is measured to shadow (docs/report/0382 has the census).
async fn declared_field(jvm: &Jvm, class_name: &str, name: &str, descriptor: &str) -> JvmResult<Box<dyn Field>> {
    let class = jvm.resolve_class(class_name).await?;
    match class.definition.field(name, descriptor, false) {
        Some(field) => Ok(field),
        None => Err(jvm
            .exception("java/lang/NoSuchFieldError", &format!("{class_name}.{name}:{descriptor}"))
            .await),
    }
}

// Unlike jvm.get_field this does not root a returned object in the caller's frame; the instance
// still holds it, so it stays reachable for as long as the caller holds the instance. `&Box` mirrors
// jvm.get_field, so a ClassInstanceRef derefs into it.
#[allow(clippy::borrowed_box)]
pub async fn get_declared_field<T>(jvm: &Jvm, instance: &Box<dyn ClassInstance>, class_name: &str, name: &str, descriptor: &str) -> JvmResult<T>
where
    T: From<JavaValue>,
{
    let field = declared_field(jvm, class_name, name, descriptor).await?;
    Ok(instance.get_field(&*field)?.into())
}

pub async fn put_declared_field<T>(
    jvm: &Jvm,
    instance: &mut Box<dyn ClassInstance>,
    class_name: &str,
    name: &str,
    descriptor: &str,
    value: T,
) -> JvmResult<()>
where
    T: Into<JavaValue>,
{
    let field = declared_field(jvm, class_name, name, descriptor).await?;
    instance.put_field(&*field, value.into())
}
