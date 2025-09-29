// Minimal window test to verify macOS window creation
use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy, NSWindow, NSWindowStyleMask, NSBackingStoreType};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use objc::{msg_send, sel, sel_impl};

fn main() {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);

        // Initialize NSApplication
        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular);

        println!("🧪 Creating minimal test window...");

        // Create window
        let window_rect = NSRect::new(NSPoint::new(200.0, 200.0), NSSize::new(400.0, 300.0));
        let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
            window_rect,
            NSWindowStyleMask::NSTitledWindowMask | NSWindowStyleMask::NSClosableWindowMask,
            NSBackingStoreType::NSBackingStoreBuffered,
            NO,
        );

        // Set title
        let title = NSString::alloc(nil).init_str("Window Test");
        window.setTitle_(title);

        // Show window with maximum force
        window.center();
        window.makeKeyAndOrderFront_(nil);
        window.orderFrontRegardless();
        window.makeMainWindow();
        window.makeKeyWindow();

        // Force app activation
        app.activateIgnoringOtherApps_(YES);

        println!("✅ Window should now be visible!");
        println!("Press any key and Enter to exit...");

        // Wait for user input
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let _: () = msg_send![pool, drain];
    }
}