package javax.microedition.rms;

public class RecordStore {
    public static RecordStore openRecordStore(String name, boolean create) throws RecordStoreException { return null; }
    public int getNumRecords() throws RecordStoreException { return 0; }
    public byte[] getRecord(int id) throws RecordStoreException { return null; }
    public int addRecord(byte[] data, int offset, int len) throws RecordStoreException { return 0; }
    public void setRecord(int id, byte[] data, int offset, int len) throws RecordStoreException {}
    public void closeRecordStore() throws RecordStoreException {}
}
