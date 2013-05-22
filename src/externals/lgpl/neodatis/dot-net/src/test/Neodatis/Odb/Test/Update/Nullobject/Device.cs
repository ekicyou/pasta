using NUnit.Framework;
namespace NeoDatis.Odb.Test.Update.Nullobject
{
	public interface Device
	{
		string GetName();

		string GetPhysicalAddress();

		string GetIpAddress();

		int GetPort();
	}
}
