using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using P2ClientGateMTA;
using System.Runtime.InteropServices;
using System.Windows.Forms;
using System.IO;

namespace new_robot_uralpro
{
    //Класс отправки транзакций Плаза2
     public class ClassP2Trans
     {
        public static bool m_stop = false;
        public static bool flood = false;
        public static double flTimeEnd = 0;
        string srvAddr = "";
        CP2BLMessage msg;
        CP2BLMessageFactory msgs;
        CP2Connection conn;
        static StreamWriter sw_logFile;
        static string pathErr = @"c:\log\" + DateTime.Now.ToString("yyMMdd") + ".P2Trans.log";
        int code = -1;

        [MTAThread]
        public void Run()
        {
            
            conn = new CP2Connection();
            conn.Host = "localhost";
            conn.Port = 4001;
            conn.AppName = "p2_trans";//"AsyncOrdSend";//
            IP2ConnectionEvent_ConnectionStatusChangedEventHandler connStatusHandler = new IP2ConnectionEvent_ConnectionStatusChangedEventHandler(ConnectionStatusChanged);
            conn.ConnectionStatusChanged += connStatusHandler;
            //conn.Connect();

            msgs = new CP2BLMessageFactory();
            msgs.Init("message.ini", "");
            msg = msgs.CreateMessageByName("FutAddOrder");

            while (!m_stop)
            {
                try
                {
                    //System.Threading.Thread.Sleep(0);
                    // создаем соединение с роутером
                    if (conn.Status == 1)
                    {
                        conn.Connect();
                        //conn.Connect2("APPNAME=p2_f2;LRPCQ_PORT=4001");
                    }
                    try
                    {
                        while (!m_stop)
                        {
                            System.Threading.Thread.Sleep(0);
                            if (!flood) readOrderAsync(conn);
                            else
                            {
                                if (Form1.timer.ElapsedMilliseconds > flTimeEnd) flood = false;
                            }
                            uint cookie;
                            conn.ProcessMessage(out cookie, 1);
                        }
                    }
                    catch (System.Runtime.InteropServices.COMException e)
                    {
                        LogWriteLine(e.Message + "   " + e.ErrorCode);
                    }
                    conn.Disconnect();
                }
                catch (System.Runtime.InteropServices.COMException e)
                {
                    LogWriteLine(e.Message + "   " + e.ErrorCode);
                }
                catch (System.Exception e)
                {
                    LogWriteLine(e.Message + "   " + e.Source);
                }
            }
        }
        [MTAThread]
        void ConnectionStatusChanged(CP2Connection conn, TConnectionStatus newStatus)
        {
            String state = "MQ connection state ";
            if ((newStatus & TConnectionStatus.CS_CONNECTION_BUSY) != 0)
            {
                state += "BUSY";
            }
            else if ((newStatus & TConnectionStatus.CS_CONNECTION_CONNECTED) != 0)
            {
                state += "CONNECTED";
            }
            else if ((newStatus & TConnectionStatus.CS_CONNECTION_DISCONNECTED) != 0)
            {
                state += "DISCONNECTED";
            }
            else if ((newStatus & TConnectionStatus.CS_CONNECTION_INVALID) != 0)
            {
                state += "INVALID";
            }

            state += " router state ";
            if ((newStatus & TConnectionStatus.CS_ROUTER_CONNECTED) != 0)
            {
                // Когда соединились - запрашиваем адрес сервера-обработчика
                state += "CONNECTED";
                srvAddr = conn.ResolveService("FORTS_SRV");
            }
            else if ((newStatus & TConnectionStatus.CS_ROUTER_DISCONNECTED) != 0)
            {
                state += "DISCONNECTED";
            }
            else if ((newStatus & TConnectionStatus.CS_ROUTER_LOGINFAILED) != 0)
            {
                state += "LOGINFAILED";
            }
            else if ((newStatus & TConnectionStatus.CS_ROUTER_NOCONNECT) != 0)
            {
                state += "NOCONNECT";
            }
            else if ((newStatus & TConnectionStatus.CS_ROUTER_RECONNECTING) != 0)
            {
                state += "RECONNECTING";
            }
            LogWriteLine(state);
        }
        
        //Считывание транзакций из хранилища cdTransP2 и отправка приказов на биржу
        [MTAThread]
        public void readOrderAsync(CP2Connection conn)
        {
            int trPos = Form1.cdTransP2.pos;
            for (int i = code + 1; i <= trPos; i++)
            {
               
                try
                {
                    string s = "";
                    //добавление ордера
                    if (Form1.cdTransP2.p2type[i] == 1)
                    {
                        string isin=Convert.ToString(ClassP2.cdFutInfo.convertShortIsinToIsin(Form1.cdTransP2.stname[i]));
                        string price = Convert.ToString(Convert.ToInt32(Form1.cdTransP2.price[i]));
                        string amount=Convert.ToString(Convert.ToInt32(Form1.cdTransP2.volume[i]));
                        string client_code = Form1.client_code_P2;
                        int operation = Form1.cdTransP2.operation[i];
                        string comment = Form1.cdTransP2.comment[i];

                        msg = msgs.CreateMessageByName("FutAddOrder");
                        msg.DestAddr = srvAddr;
                        msg.set_Field("P2_Category", "FORTS_MSG");
                        msg.set_Field("P2_Type", 1);
                        msg.set_Field("isin",isin);
                        msg.set_Field("price", price);
                        msg.set_Field("amount", amount);
                        msg.set_Field("client_code", client_code);
                        msg.set_Field("type", 1);
                        msg.set_Field("dir", operation);
                        msg.set_Field("comment", comment);

                        //s = "P2_Type=1(ADD); isin= " + isin + "; price=" + price + " ; amount=" + amount +
                        //" ; client_code=" + client_code + " ; dir=" + operation + " ; comment=" + comment;
                    }
                    //передвижение ордера
                    else if (Form1.cdTransP2.p2type[i] == 5)//move
                    {
                        long ordNum = Form1.cdTransP2.ordNum[i];
                        string price = Convert.ToString(Convert.ToInt32(Form1.cdTransP2.price[i]));

                        msg = msgs.CreateMessageByName("FutMoveOrder");
                        msg.DestAddr = srvAddr;
                        msg.set_Field("P2_Category", "FORTS_MSG");
                        msg.set_Field("P2_Type", 5);
                        msg.set_Field("regime", "0");
                        msg.set_Field("order_id1", ordNum);
                        msg.set_Field("price1", price);
                        msg.set_Field("amount1", "1");

                        //s = "P2_Type=5(MOVE); ordNum= " + ordNum + "; price=" + price;
                    }
                    //удаление ордера
                    else if (Form1.cdTransP2.p2type[i] == 2)//del
                    {
                        long ordNum = Form1.cdTransP2.ordNum[i];

                        msg = msgs.CreateMessageByName("FutDelOrder");
                        msg.DestAddr = srvAddr;
                        msg.set_Field("P2_Category", "FORTS_MSG");
                        msg.set_Field("P2_Type", 2);
                        msg.set_Field("order_id", ordNum);

                        //s = "P2_Type=2(DEL); ordNum= " + ordNum;
                    }
                    
                    //Посылка приказа
                    msg.SendAsync2(conn, 150000, new CP2SendAsync2Event(),Form1.cdTransP2.amTrans[i]);
                    Form1.cdTransP2.trTime[i] = Form1.timer.ElapsedMilliseconds;//DateTime.Now.TimeOfDay;
                    //ClassP2Trans.LogWrite((Form1.dt + new TimeSpan(0, 0, 0, 0, (int)(Form1.timer.ElapsedMilliseconds))).ToString("HH:mm:ss.fff")); //Лог включен/выключен
                    //LogWriteLine(Form1.cdTransP2.amTrans[i] + " " + s);  //Лог включен/выключен
                }
                catch (System.Exception e)
                {
                    Form1.cbTrD.insertData((uint)Form1.cdTransP2.amTrans[i], 0, "", 1,Form1.timer.ElapsedMilliseconds,0);
                    LogWriteLine(e.Message + "   " + e.Source);
                }
            }
            code =trPos;
        }
        
         //Запись лога
        [MTAThread]
        public static void LogWriteLine(string s)
        {
            if (sw_logFile == null)
            {
                sw_logFile = new StreamWriter(pathErr, true, System.Text.Encoding.Unicode);
            }
            sw_logFile.WriteLine("   " + s);
            sw_logFile.Flush();
        }
        
         //Запись лога
        [MTAThread]
        public static void LogWrite(string s)
        {
            if (sw_logFile == null)
            {
                sw_logFile = new StreamWriter(pathErr, true, System.Text.Encoding.Unicode);
            }
            DateTime dt = DateTime.Now;
            sw_logFile.Write(s + "   ");
            sw_logFile.Flush();
        }
    }

     //Получение коллбэка
    [ComVisible(true)]
    public class CP2SendAsync2Event : IP2AsyncSendEvent2
    {
        [MTAThread]
        void IP2AsyncSendEvent2.SendAsync2Reply(CP2BLMessage reply, uint errCode, long eventParam)
        {
            try
            {
                int replyType = Convert.ToInt32(reply.get_Field("P2_Type"));
                string s="";
                if (replyType == 105) //move
                {
                    Form1.cbTrD.transactionID[Form1.cbTrD.pos + 1] = (uint)eventParam;
                    Form1.cbTrD.orderNumber[Form1.cbTrD.pos + 1] = Convert.ToDouble(reply.get_Field("order_id1"));
                    Form1.cbTrD.transactionReplyMessage[Form1.cbTrD.pos + 1] = Convert.ToString(reply.get_Field("message"));
                    Form1.cbTrD.cbtime[Form1.cbTrD.pos + 1] = Form1.timer.ElapsedMilliseconds;// DateTime.Now;
                    Form1.cbTrD.transactionResult[Form1.cbTrD.pos + 1] = 0;
                    Form1.cbTrD.pos++;

                    //s = eventParam + "     message= " + reply.get_Field("message") + "; ordNum= " + Convert.ToString(reply.get_Field("order_id1"));
                }
                else if (replyType == 101) //add
                {
                    Form1.cbTrD.transactionID[Form1.cbTrD.pos + 1] = (uint)eventParam;
                    Form1.cbTrD.orderNumber[Form1.cbTrD.pos + 1] = Convert.ToDouble(reply.get_Field("order_id"));
                    Form1.cbTrD.transactionReplyMessage[Form1.cbTrD.pos + 1] = Convert.ToString(reply.get_Field("message"));
                    Form1.cbTrD.cbtime[Form1.cbTrD.pos + 1] = Form1.timer.ElapsedMilliseconds;//DateTime.Now;
                    Form1.cbTrD.transactionResult[Form1.cbTrD.pos + 1] = 0;
                    Form1.cbTrD.pos++;

                    //s = eventParam + "     message= " + reply.get_Field("message") + "; ordNum= " +Convert.ToString(reply.get_Field("order_id"));
                }
                else if (replyType == 102) //kill
                {
                    Form1.cbTrD.transactionID[Form1.cbTrD.pos + 1] = (uint)eventParam;
                    //FormStartClass.cdQuikCallBack.orderNumber[FormStartClass.cdQuikCallBack.pos + 1] = Convert.ToDouble(reply.get_Field("order_id"));
                    Form1.cbTrD.transactionReplyMessage[Form1.cbTrD.pos + 1] = Convert.ToString(reply.get_Field("message"));
                    Form1.cbTrD.cbtime[Form1.cbTrD.pos + 1] = Form1.timer.ElapsedMilliseconds;//DateTime.Now;
                    Form1.cbTrD.transactionResult[Form1.cbTrD.pos + 1] = 0;
                    Form1.cbTrD.delamount[Form1.cbTrD.pos + 1] = Convert.ToInt32(reply.get_Field("amount"));
                    Form1.cbTrD.pos++;

                    //s = eventParam + "     message= " + reply.get_Field("message");// +"; ordNum= " + Convert.ToString(reply.get_Field("order_id"));
                }
                else
                {
                    //Определение флуда и таймаут
                    if (replyType == 99) //flud
                    {
                        Form1.cbTrD.insertData((uint)eventParam, 0, "flood", 0, Form1.timer.ElapsedMilliseconds, 0);
                        int pen = Convert.ToInt32(reply.get_Field("penalty_remain"));
                        s = eventParam + "     message=" + reply.get_Field("message") + "   penalty=" + pen + ";";
                        ClassP2Trans.LogWriteLine(s);
                        ClassP2Trans.flood = true;
                        ClassP2Trans.flTimeEnd = Form1.timer.ElapsedMilliseconds + pen + 10;
                    }
                    else
                    {
                        string mes = "unknown  " + replyType;
                        Form1.cbTrD.insertData((uint)eventParam, 0, mes, 0, Form1.timer.ElapsedMilliseconds, 0);
                        s = eventParam + "  unknown  " + replyType;//eventParam + " code= " + Convert.ToString(reply.get_Field("code")) + "     replyType= " + replyType + "     message= " + Convert.ToString(reply.get_Field("message"));
                        ClassP2Trans.LogWriteLine(s);
                    }
                }
                //ClassP2Trans.LogWrite((Form1.dt + new TimeSpan(0, 0, 0, 0, (int)(Form1.timer.ElapsedMilliseconds))).ToString("HH:mm:ss.fff")); ClassP2Trans.LogWriteLine(s); //Лог включен/выключен
            }
            catch (System.Exception e)
            {
                ClassP2Trans.LogWriteLine(e.Message + "   " + e.Source);
            }
        }
    }
}
