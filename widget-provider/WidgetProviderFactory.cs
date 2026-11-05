using System.Runtime.InteropServices;
using Microsoft.Windows.Widgets.Providers;
using WinRT;

namespace AiUsageDock.Widgets;

internal static class ComGuids
{
    public const string ClassFactory = "00000001-0000-0000-C000-000000000046";
}

[ComImport]
[ComVisible(false)]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid(ComGuids.ClassFactory)]
internal interface IClassFactory
{
    [PreserveSig]
    int CreateInstance(IntPtr pUnkOuter, ref Guid riid, out IntPtr ppvObject);

    [PreserveSig]
    int LockServer(bool fLock);
}

[ComVisible(true)]
internal sealed class WidgetProviderFactory<T> : IClassFactory
    where T : IWidgetProvider, new()
{
    private const int ClassENoAggregation = -2147221232;

    public int CreateInstance(IntPtr pUnkOuter, ref Guid riid, out IntPtr ppvObject)
    {
        ppvObject = IntPtr.Zero;

        if (pUnkOuter != IntPtr.Zero)
        {
            return ClassENoAggregation;
        }

        IntPtr provider = IntPtr.Zero;
        try
        {
            provider = MarshalInspectable<IWidgetProvider>.FromManaged(new T());
            return Marshal.QueryInterface(provider, in riid, out ppvObject);
        }
        catch (Exception exception)
        {
            Log.Error("CreateInstance failed", exception);
            return Marshal.GetHRForException(exception);
        }
        finally
        {
            if (provider != IntPtr.Zero)
            {
                Marshal.Release(provider);
            }
        }
    }

    int IClassFactory.LockServer(bool fLock) => 0;
}
