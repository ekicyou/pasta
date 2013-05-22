namespace NeoDatis.Odb.Core.Layers.Layer3
{
	/// <summary>
	/// <p>
	/// An interface for refactoring
	/// </p>
	/// </summary>
	public interface IRefactorManager
	{
		/// <exception cref="System.IO.IOException"></exception>
		void RenameClass(string className, string newClassName);

		void RemoveClass(string className);

		/// <exception cref="System.IO.IOException"></exception>
		void RenameField(string className, string attributeName, string newAttributeName);

		void AddField(string className, System.Type fieldType, string fieldName);

		/// <exception cref="System.IO.IOException"></exception>
		void RemoveField(string className, string attributeName);

		void ChangeFieldType(string className, string attributeName, System.Type newType);
	}
}
