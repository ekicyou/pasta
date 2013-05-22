namespace NeoDatis.Odb.Test.Query.Values
{
	/// <author>olivier</author>
	public class Handler
	{
		private System.Collections.IList parameters;

		public Handler() : base()
		{
			this.parameters = new System.Collections.ArrayList();
		}

		public virtual System.Collections.IList GetListOfParameters()
		{
			return parameters;
		}

		public virtual void SetListOfParameters(System.Collections.IList listOfParameters
			)
		{
			this.parameters = listOfParameters;
		}

		/// <param name="parameter"></param>
		public virtual void AddParameter(NeoDatis.Odb.Test.Query.Values.Parameter parameter
			)
		{
			parameters.Add(parameter);
		}
	}
}
