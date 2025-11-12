using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Windows.Forms;

namespace new_robot_uralpro
{
    public partial class FormOutput : Form
    {
      
        Button btnStop = new Button();
        public static bool stopped = false;
        OutBlock[] ob = new OutBlock[Form1.nThr];


        public FormOutput()
        {
            InitializeComponent();
        }

        public void formStart()
        {
            StartPosition = FormStartPosition.Manual;
            Location = new Point(1000, 270);
            Size = new Size(280,50+142*(Form1.nThr));
            for (int i = 0; i < Form1.nThr; i++)
            {
                ob[i] = new OutBlock();
                ob[i].Location = new Point(1, 30+i * 142);
                ob[i].addElem(Form1.res[i].id, i);
                //ob[i].groupBox1.Text = Form1.res[i].id;
                //ob[i].button1.Name = "ab_" + i;
                //ob[i].button2.Name = "ps_" + i;
                Controls.Add(ob[i]);
            }
            btnStop.Text = "abort";
            btnStop.Size = new Size(70, 25);
            btnStop.Click += new EventHandler(Form1.btnStop_Clicked);
            btnStop.Location = new Point(180, 5);
            Controls.Add(btnStop);
            TopMost = true;
            Show();
            while (!stopped)
            {
                Application.DoEvents();
                for (int i = 0; i < Form1.nThr; i++)
                {
                    ob[i].addData(Form1.res[i].summ, Form1.res[i].futpoz, Form1.res[i].trans, Form1.res[i].trades,
                        Form1.res[i].delay, Form1.res[i].sumMax, Form1.res[i].sumMin, Form1.res[i].contracts,
                        Form1.res[i].err, Form1.res[i].time.Hours, Form1.res[i].time.Minutes, Form1.res[i].time.Seconds);
                    //ob[i].label10.Text = String.Format("{0:F1}", Form1.res[i].summ);
                    //ob[i].label9.Text = String.Format("{0}", Form1.res[i].futpoz);
                    //ob[i].label8.Text = String.Format("{0}/{1}", Form1.res[i].trans, Form1.res[i].trades);
                    //ob[i].label6.Text = String.Format("{0:F2}", Form1.res[i].delay);
                    //ob[i].label20.Text = String.Format("{0:F1}", Form1.res[i].sumMax);
                    //ob[i].label19.Text = String.Format("{0:F1}", Form1.res[i].sumMin);
                    //ob[i].label18.Text = String.Format("{0}", Form1.res[i].contracts);
                    //ob[i].label17.Text = String.Format("{0}", Form1.res[i].err);
                    //ob[i].label11.Text = String.Format("{0}:{1}:{2}", Form1.res[i].time.Hours, Form1.res[i].time.Minutes,
                    //    Form1.res[i].time.Seconds);
                }
                System.Threading.Thread.Sleep(1000);
            }
        }

    }
}
