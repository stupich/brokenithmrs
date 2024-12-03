use jni::{
    self,
    objects::{JObject, JValueGen},
};

pub fn hide_ui() {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };
    let mut env = vm.attach_current_thread().unwrap();
    let activity_class = env.find_class("android/app/NativeActivity").unwrap();
    let window = env
        .call_method(context, "getWindow", "()Landroid/view/Window;", &[])
        .unwrap()
        .l()
        .unwrap();

    let decor_view = env
        .call_method(window, "getDecorView", "()Landroid/view/View;", &[])
        .unwrap()
        .l()
        .unwrap();

    let controller = env
        .call_method(
            decor_view,
            "getWindowInsetsController",
            "()Landroid/view/WindowInsetsController;",
            &[],
        )
        .unwrap()
        .l()
        .unwrap();

    let val = 1 << 0 | 1 << 1 | 1 << 2;
    let jval = JValueGen::Int(val);

    let _ = env.call_method(controller, "hide", "(I)V", &[jval]);
}

pub fn show_soft_input(show: bool) -> bool {
    use jni::objects::JValue;

    let ctx = ndk_context::android_context();
    let vm = match unsafe { jni::JavaVM::from_raw(ctx.vm() as _) } {
        Ok(value) => value,
        Err(e) => {
            return false;
        }
    };
    let activity = unsafe { jni::objects::JObject::from_raw(ctx.context() as _) };
    let mut env = match vm.attach_current_thread() {
        Ok(value) => value,
        Err(e) => {
            return false;
        }
    };

    let mut envs = match vm.attach_current_thread() {
        Ok(value) => value,
        Err(e) => {
            return false;
        }
    };

    let class_ctxt = match env.find_class("android/content/Context") {
        Ok(value) => value,
        Err(e) => {
            return false;
        }
    };
    let ims = match env.get_static_field(class_ctxt, "INPUT_METHOD_SERVICE", "Ljava/lang/String;") {
        Ok(value) => value,
        Err(e) => {
            return false;
        }
    };

    let im_manager = match env
        .call_method(
            &activity,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[ims.borrow()],
        )
        .unwrap()
        .l()
    {
        Ok(value) => value,
        Err(e) => {
            return false;
        }
    };

    let jni_window = match env
        .call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])
        .unwrap()
        .l()
    {
        Ok(value) => value,
        Err(e) => {
            return false;
        }
    };
    let view = match env
        .call_method(jni_window, "getDecorView", "()Landroid/view/View;", &[])
        .unwrap()
        .l()
    {
        Ok(value) => value,
        Err(e) => {
            return false;
        }
    };
    if show {
        let result = env
            .call_method(
                im_manager,
                "showSoftInput",
                "(Landroid/view/View;I)Z",
                &[JValue::Object(&view), 0i32.into()],
            )
            .unwrap()
            .z()
            .unwrap();
        result
    } else {
        let window_token = env
            .call_method(view, "getWindowToken", "()Landroid/os/IBinder;", &[])
            .unwrap()
            .l()
            .unwrap();
        let jvalue_window_token = jni::objects::JValueGen::Object(&window_token);

        let result = env
            .call_method(
                im_manager,
                "hideSoftInputFromWindow",
                "(Landroid/os/IBinder;I)Z",
                &[jvalue_window_token, 0i32.into()],
            )
            .unwrap()
            .z()
            .unwrap();
        result
    }
}
