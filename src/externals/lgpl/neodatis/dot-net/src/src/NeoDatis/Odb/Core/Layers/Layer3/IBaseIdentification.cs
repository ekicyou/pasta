namespace NeoDatis.Odb.Core.Layers.Layer3
{
	/// <summary>An interface to get info about database parameters</summary>
	/// <author>osmadja</author>
	public interface IBaseIdentification
	{
		bool CanWrite();

		string GetIdentification();

		bool IsNew();

		bool IsLocal();

		string GetDirectory();

		string GetUserName();

		string GetPassword();
	}
}
