using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using NDde.Client;
using System.Windows.Forms;
using System.IO;
using System.Globalization;

namespace new_robot_uralpro
{
    class ClassDdeClient
    {
        static bool compl = false;
        string ddeServerName = "QST";
        public string[] ftName;
        static string cultureName = "en-US";
        StreamWriter sw_logFile;
        string DDELog = "DDEdata.csv";
        double[] price;
        TimeSpan time;
        CultureInfo culture = new CultureInfo(cultureName);
        DdeClient[] client;

       
        public void mainDdeClient()
        {

            int Ni = ftName.Length;
            client = new DdeClient[Ni];
            price = new double[Ni];

            using (new StreamWriter(DDELog, false, System.Text.Encoding.Unicode)){}

            using (client[0])
            {

                for (int i = 0; i < ftName.Length; i++)
                {
                    client[i] = new DdeClient(ddeServerName, ftName[i] + "_Ticks(0)");
                    client[i].Disconnected += OnDisconnected;
                    client[i].Connect();
                    client[i].Execute("GET", 60000);
                    client[i].BeginRequest("ls", 1, OnRequestComplete, client[i]);
                    client[i].StartAdvise("ls", 1, true, 60000);
                    client[i].StartAdvise("ts", 1, true, 60000);
                    client[i].Advise += OnAdvise;
                    System.Threading.Thread.Sleep(10);
                }
                System.Threading.Thread.Sleep(System.Threading.Timeout.Infinite);

            }
        }
       
        private void OnAdvise(object sender, DdeAdviseEventArgs args)
        {
            for (int i = 0; i < client.Length; i++)
            {
                if (sender.Equals(client[i]))
                {
                    if (args.Item == "ls") price[i] = Convert.ToDouble(args.Text, culture);
                    if (args.Item == "ts")
                    {
                        if (ftName[i].Substring(0, 3) == "ECL") time = Convert.ToDateTime(args.Text, culture).AddHours(8).TimeOfDay;
                        else time = Convert.ToDateTime(args.Text, culture).AddHours(9).TimeOfDay;
                        Form1.currDataInd.insertData(-1, ftName[i], "", price[i], 0, time, 0, 0, "", 0);
                        //LogWriteLine(price[i] + ";" + ftName[i] + ";" + time.ToString());
                    }
                }
            }
        }
      
        private void OnRequestComplete(IAsyncResult ar)
        {
            try
            {
                DdeClient client = (DdeClient)ar.AsyncState;
                byte[] data = client.EndRequest(ar);
                LogWriteLine("OnRequestComplete: " + Encoding.ASCII.GetString(data));
                //MessageBox.Show("OnRequestComplete: " + Encoding.ASCII.GetString(data));
            }
            catch (Exception e)
            {
                MessageBox.Show("OnRequestComplete: " + e.Message);
            }
        }

        private void OnDisconnected(object sender, DdeDisconnectedEventArgs args)
        {
            MessageBox.Show(
                "OnDisconnected: " +
                "IsServerInitiated=" + args.IsServerInitiated.ToString() + " " +
                "IsDisposed=" + args.IsDisposed.ToString());
        }

        public void LogWriteLine(string s)
        {
            if (sw_logFile == null)
            {
                sw_logFile = new StreamWriter(DDELog, true, System.Text.Encoding.Unicode);
            }
            sw_logFile.WriteLine(s);
            sw_logFile.Flush();
        }

    }
}