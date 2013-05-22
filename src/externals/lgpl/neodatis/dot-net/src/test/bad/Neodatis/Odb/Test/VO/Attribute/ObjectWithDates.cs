namespace NeoDatis.Odb.Test.VO.Attribute
{
	/// <author>olivier</author>
	public class ObjectWithDates
	{
		private string name;

		private System.DateTime javaUtilDate;

		private System.DateTime javaSqlDte;

		private System.DateTime timestamp;

		public ObjectWithDates(string name, System.DateTime javaUtilDate, System.DateTime
			 javaSqlDte, System.DateTime timestamp) : base()
		{
			this.name = name;
			this.javaUtilDate = javaUtilDate;
			this.javaSqlDte = javaSqlDte;
			this.timestamp = timestamp;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual System.DateTime GetJavaUtilDate()
		{
			return javaUtilDate;
		}

		public virtual void SetJavaUtilDate(System.DateTime javaUtilDate)
		{
			this.javaUtilDate = javaUtilDate;
		}

		public virtual System.DateTime GetJavaSqlDte()
		{
			return javaSqlDte;
		}

		public virtual void SetJavaSqlDte(System.DateTime javaSqlDte)
		{
			this.javaSqlDte = javaSqlDte;
		}

		public virtual System.DateTime GetTimestamp()
		{
			return timestamp;
		}

		public virtual void SetTimestamp(System.DateTime timestamp)
		{
			this.timestamp = timestamp;
		}
	}
}
