package javax.microedition.midlet;

// Compile-time stub only — never packaged. The engine supplies the real class.
public abstract class MIDlet {
    protected MIDlet() {}
    protected abstract void startApp() throws MIDletStateChangeException;
    protected abstract void pauseApp();
    protected abstract void destroyApp(boolean unconditional) throws MIDletStateChangeException;
    public final void notifyDestroyed() {}

    public final String getAppProperty(String key) { return null; }
}
