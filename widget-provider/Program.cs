using System.Runtime.InteropServices;
using WinRT;

namespace AiUsageDock.Widgets;

internal static class Program
{
    private const string ProviderClsid = "3F2C9A7E-6B1D-4E8A-9C5B-2D7F4A1C8E30";

    [DllImport("ole32.dll")]
    private static extern int CoRegisterClassObject(
        [MarshalAs(UnmanagedType.LPStruct)] Guid rclsid,
        [MarshalAs(UnmanagedType.IUnknown)] object pUnk,
        uint dwClsContext,
        uint flags,
        out uint lpdwRegister);

    [DllImport("ole32.dll")]
    private static extern int CoRevokeClassObject(uint dwRegister);

    [STAThread]
    private static void Main()
    {
        ComWrappersSupport.InitializeComWrappers();

        var factory = new WidgetProviderFactory<UsageWidgetProvider>();
        Marshal.ThrowExceptionForHR(
            CoRegisterClassObject(Guid.Parse(ProviderClsid), factory, 0x4, 0x1, out var cookie));

        try
        {
            UsageWidgetProvider.WaitForExit();
        }
        finally
        {
            CoRevokeClassObject(cookie);
            GC.KeepAlive(factory);
        }
    }
}
