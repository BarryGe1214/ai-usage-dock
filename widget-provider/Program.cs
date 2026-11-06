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

    [MTAThread]
    private static int Main(string[] args)
    {
        ComWrappersSupport.InitializeComWrappers();

        if (args.Contains("--check-runtime"))
        {
            try
            {
                using var managerFactory = ActivationFactory.Get(
                    "Microsoft.Windows.Widgets.Providers.WidgetManager",
                    new Guid("00000035-0000-0000-C000-000000000046"));
                _ = new Microsoft.Windows.Widgets.Providers.WidgetUpdateRequestOptions("runtime-check");
                return 0;
            }
            catch (Exception exception)
            {
                Log.Error("Widgets runtime activation check failed", exception);
                return 1;
            }
        }

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

        return 0;
    }
}
