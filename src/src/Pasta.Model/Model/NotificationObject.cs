using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using System.Threading.Tasks;

namespace Pasta.Model
{
    public abstract class NotificationObject : INotifyPropertyChanged
    {
        protected NotificationObject() { }

        public event PropertyChangedEventHandler PropertyChanged;
        internal void OnPropertyChanged([CallerMemberName] string propertyName = null)
        {
            var h = this.PropertyChanged;
            if (h != null) {
                h(this, new PropertyChangedEventArgs(propertyName));
            }
        }

    }

}