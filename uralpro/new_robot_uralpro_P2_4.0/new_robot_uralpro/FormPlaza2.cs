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
    public partial class FormPlaza2 : Form
    {
        public FormPlaza2()
        {
            InitializeComponent();
        }

        public void addBS1(BindingSource bs)
        {
            dataGridView1.DataSource = bs;
        }
        public void addBS2(BindingSource bs)
        {
            dataGridView2.DataSource = bs;
        }
        public void addBS3(BindingSource bs)
        {
            dataGridView3.DataSource = bs;
        }
        public void addBS4(BindingSource bs)
        {
            dataGridView4.DataSource = bs;
        }

        public void addLbls(string servTime,string tradesState,string aggrState)
        {
            label1.Text = servTime;
            label3.Text = tradesState;
            label12.Text = aggrState;
        }

     
        private void button1_Click(object sender, EventArgs e)
        {
            if (MessageBox.Show("закрыть P2?", "", MessageBoxButtons.YesNo) == System.Windows.Forms.DialogResult.Yes)
            {
                ClassP2.m_stop = true;
                ClassP2Trans.m_stop = true;
            }
        }
      
        public void button2_Click(object sender, EventArgs e)
        {
            int operation;
            string comment = textBox3.Text;
            if (textBox2.Text == "b") operation = 1;
            else operation = 2;
            int am = -1;
            Form1.cdTransP2.addOrder(textBox1.Text, operation, Convert.ToDouble(textBox5.Text), Convert.ToDouble(textBox4.Text), comment,am);
        }
      
        private void button4_Click(object sender, EventArgs e)
        {
            int am = -1;
            Form1.cdTransP2.DelOrder((long)Convert.ToUInt64(this.textBox8.Text), am);
        }
       
        private void button3_Click(object sender, EventArgs e)
        {
            int am = -1;
            Form1.cdTransP2.moveOrder((long)Convert.ToUInt64(this.textBox6.Text), Convert.ToDouble(textBox7.Text), am);
        }

    }
}
