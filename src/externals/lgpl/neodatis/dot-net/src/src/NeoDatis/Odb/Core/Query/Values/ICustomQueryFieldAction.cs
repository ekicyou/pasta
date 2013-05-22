namespace NeoDatis.Odb.Core.Query.Values
{
	/// <summary>Used to implement custom query action.</summary>
	/// <remarks>Used to implement custom query action.</remarks>
	/// <author>osmadja</author>
	public interface ICustomQueryFieldAction : NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction
	{
		void SetAttributeName(string attributeName);

		void SetAlias(string alias);
	}
}
