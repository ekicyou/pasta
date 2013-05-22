namespace NeoDatis.Odb.Test.Newbie.VO
{
	/// <summary>It is just a simple test to help the newbies</summary>
	/// <author>mayworm at <xmpp://mayworm@gmail.com></author>
	public class Car
	{
		private string name;

		private int numberOfOccupant;

		private string model;

		private NeoDatis.Odb.Test.Newbie.VO.Driver driver;

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual int GetNumberOfOccupant()
		{
			return numberOfOccupant;
		}

		public virtual void SetNumberOfOccupant(int numberOfOccupant)
		{
			this.numberOfOccupant = numberOfOccupant;
		}

		public virtual string GetModel()
		{
			return model;
		}

		public virtual void SetModel(string model)
		{
			this.model = model;
		}

		public Car(string name, int numberOfOccupant, string model)
		{
			this.name = name;
			this.numberOfOccupant = numberOfOccupant;
			this.model = model;
		}

		public Car(string name, int numberOfOccupant, string model, NeoDatis.Odb.Test.Newbie.VO.Driver
			 driver)
		{
			this.name = name;
			this.numberOfOccupant = numberOfOccupant;
			this.model = model;
			this.driver = driver;
		}

		public virtual NeoDatis.Odb.Test.Newbie.VO.Driver GetDriver()
		{
			return driver;
		}

		public virtual void SetDriver(NeoDatis.Odb.Test.Newbie.VO.Driver driver)
		{
			this.driver = driver;
		}
	}
}
