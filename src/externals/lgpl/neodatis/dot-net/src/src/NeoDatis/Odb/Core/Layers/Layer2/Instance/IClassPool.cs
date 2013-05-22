namespace NeoDatis.Odb.Core.Layers.Layer2.Instance
{
	public interface IClassPool
	{
		System.Type GetClass(string className);

		System.Reflection.ConstructorInfo GetConstrutor(string className);

		void AddConstrutor(string className, System.Reflection.ConstructorInfo constructor
			);

		void Reset();
	}
}
