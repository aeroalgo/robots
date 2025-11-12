using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Drawing;
using System.Data;
using System.Linq;
using System.Text;
using System.Windows.Forms;

namespace new_robot_uralpro
{
    public partial class OutBlock : UserControl
    {
        public OutBlock()
        {
            InitializeComponent();
        }

        public void addData(double summ, int futpoz,int trans,int trades,double delay, double sumMax, double sumMin,
            int contracts, int err, int Hours, int Minutes,int Seconds)
        {
            label10.Text = String.Format("{0:F1}", summ);
            label9.Text = String.Format("{0}", futpoz);
            label8.Text = String.Format("{0}/{1}", trans, trades);
            label6.Text = String.Format("{0:F2}", delay);
            label20.Text = String.Format("{0:F1}", sumMax);
            label19.Text = String.Format("{0:F1}", sumMin);
            label18.Text = String.Format("{0}", contracts);
            label17.Text = String.Format("{0}", err);
            label11.Text = String.Format("{0}:{1}:{2}", Hours,Minutes,Seconds);
        }

        public void addElem(string id, int i)
        {
            groupBox1.Text = id;
            button1.Name = "ab_" + i;
            button2.Name = "ps_" + i;
        }
        public void button1_Click(object sender, EventArgs e)
        {
           Button bt = (Button)sender;
           string[] st = bt.Name.Split('_');
           int n = Convert.ToInt32(st[1]);
           Form1.classDispose(n);
           bt.Enabled = false;
           OutBlock ob =(OutBlock)bt.Parent;
           ob.Enabled = false;
        }

        public void button2_Click(object sender, EventArgs e)
        {
            bool status;
            Button bt = (Button)sender;
            OutBlock ob = (OutBlock)bt.Parent;
            if (bt.Text == "pl")
            {
                status = false;
                bt.Text = "ps";
                ob.groupBox1.Enabled = true;

            }
            else
            {
                status = true;
                bt.Text = "pl";
                ob.groupBox1.Enabled = false;
            }
            string[] st = bt.Name.Split('_');
            int n = Convert.ToInt32(st[1]);
            Form1.classPause(n,status);
        }
    }
}
