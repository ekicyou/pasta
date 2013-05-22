namespace NeoDatis.Odb.Core
{
	public interface IError
	{
		NeoDatis.Odb.Core.IError AddParameter(object o);

		NeoDatis.Odb.Core.IError AddParameter(string s);

		NeoDatis.Odb.Core.IError AddParameter(int i);

		NeoDatis.Odb.Core.IError AddParameter(byte i);

		NeoDatis.Odb.Core.IError AddParameter(long l);
	}
}
