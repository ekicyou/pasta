using System;
using System.Windows;
using System.Windows.Threading;
using System.Collections.ObjectModel;
using System.ComponentModel;
using TableView;

namespace TableViewDemo
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        const int RowCount = 1000;      // number of rows to generate for the TableView
        private Random rand = new Random();
        private DispatcherTimer Timer = new DispatcherTimer();  // used to show some dynamic changes to the row data

        public ObservableCollection<Row> MyRows { get; set; }

        // Add some row data for the TableView
        private void BuildRows()
        {
            MyRows = new ObservableCollection<Row>();

            for (int i = 0; i < RowCount; ++i)
                MyRows.Add(new Row(i.ToString()));
        }

        // Programatically add some columns to the TableView
        private void BuildTableViewColumns()
        {
            for (int i = 0; i < 26; ++i)
            {
                var path = "Data[" + i.ToString() + "]";

                tableView.Columns.Add(new TableViewColumn() { Title = path, Width = 60, ContextBindingPath = path });
            }
        }

        public MainWindow()
        {
            InitializeComponent();
            DataContext = this;

            BuildRows();
            BuildTableViewColumns();

            // randomly change the state of some fields in the first 100 rows
            Timer.Interval = new TimeSpan(100000);
            Timer.Tick += (s, e) =>
            {
                MyRows[rand.Next(100)].Data[rand.Next(25)].State = rand.Next(3);
            };

            Timer.Start();
        }

    }


    //------------------------------------------------------------------
    // Field class used to hold data in an observable object so that the
    // visuals will update when the state changes.
    //------------------------------------------------------------------
    public class Field : INotifyPropertyChanged
    {
        public event PropertyChangedEventHandler PropertyChanged;

        private int _state;
        public int State
        {
            get { return _state; }
            set
            {
                _state = value;
                NotifyPropertyChanged("State");
            }
        }

        private string _fieldValue;
        public string FieldValue
        {
            get { return _fieldValue; }
            set
            {
                _fieldValue = value;
                NotifyPropertyChanged("FieldValue");
            }
        }

        private void NotifyPropertyChanged(string name)
        {
            if (PropertyChanged != null)
                PropertyChanged(this, new PropertyChangedEventArgs(name));
        }
    }

    //------------------------------------------------------------------
    // Row class holds all the fields in an observable collection
    // plus a couple of simple properties for the xaml column binding
    //------------------------------------------------------------------
    public class Row
    {
        public string AAA { get; set; }
        public int BBB { get; set; }

        public ObservableCollection<Field> Data { get; set; }

        public Row(string id)
        {
            AAA = id;
            BBB = int.Parse( id );

            Data = new ObservableCollection<Field>();

            for (char i = 'A'; i <= 'Z'; i++)
            {
                var f = new Field() { FieldValue = i + id };
                Data.Add(f);
            }
        }
    }

}