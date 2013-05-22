using NUnit.Framework;
namespace NeoDatis.Odb.Test.VO.Human
{
	public class Animal
	{
		protected string specie;

		protected string sex;

		protected string name;

		public Animal(string specie, string sex, string name) : base()
		{
			this.specie = specie;
			this.sex = sex;
			this.name = name;
		}

		public virtual string GetSpecie()
		{
			return specie;
		}

		protected virtual void SetSpecie(string specie)
		{
			this.specie = specie;
		}

		public virtual string GetSex()
		{
			return sex;
		}

		protected virtual void SetSex(string sex)
		{
			this.sex = sex;
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}
	}
}
