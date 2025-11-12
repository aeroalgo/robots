using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace new_robot_uralpro
{
    class ClassFunc
    {
        static object flock = new object();
        static object loglock = new object();

        //Импорт библиотек API Квика
        [DllImport("TRANS2QUIK", EntryPoint = "_TRANS2QUIK_CONNECT@16")]
        public static extern int TRANS2QUIK_CONNECT(string lpcstrConnectionParamsString, ref Int32 pnExtendedErrorCode,
            string lpstrErrorMessage, UInt32 dwErrorMessageSize);
        [DllImport("TRANS2QUIK", EntryPoint = "_TRANS2QUIK_IS_DLL_CONNECTED@12")]
        public static extern int TRANS2QUIK_IS_DLL_CONNECTED(ref Int32 pnExtendedErrorCode,
            string lpstrErrorMessage, UInt32 dwErrorMessageSize);


        [DllImport("TRANS2QUIK", EntryPoint = "_TRANS2QUIK_SEND_ASYNC_TRANSACTION@16")]
        public static extern Int32 TRANS2QUIK_SEND_ASYNC_TRANSACTION([MarshalAs(UnmanagedType.LPStr)]string lpstTransactionString,
            ref Int32 pnExtendedErrorCode, string lpstErrorMessage, UInt32 dwErrorMessageSize);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_SEND_SYNC_TRANSACTION@36",
             CallingConvention = CallingConvention.StdCall)]
        static extern long TRANS2QUIK_SEND_SYNC_TRANSACTION(
            string lpstTransactionString,
            ref Int32 pnReplyCode,
            ref int pdwTransId,
            ref double pdOrderNum,
            [MarshalAs(UnmanagedType.LPStr)] StringBuilder lpstrResultMessage,
            UInt32 dwResultMessageSize,
            ref Int32 pnExtendedErrorCode,
            byte[] lpstrErrorMessage,
            UInt32 dwErrorMessageSize);

        public delegate void TransactionReplyCallback(int transactionResult, int transactionExtendedErrorCode,
            int transactionReplyCode, uint transactionID, double orderNumber, string transactionReplyMessage);

        [DllImport("TRANS2QUIK", EntryPoint = "_TRANS2QUIK_SET_TRANSACTIONS_REPLY_CALLBACK@16")]
        public static extern Int32 SetTransactionReplyCallback(TransactionReplyCallback transactionReplyCallback,
        ref Int32 extendedErrorCode, string errorMessage, UInt32 errorMessageSize);

        #region trades
        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_SUBSCRIBE_TRADES@8",
           CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 subscribe_trades([MarshalAs(UnmanagedType.LPStr)]string class_code, [MarshalAs(UnmanagedType.LPStr)]string sec_code);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_UNSUBSCRIBE_TRADES@0",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 ubsubscribe_trades();

        public delegate void trade_status_callback(
                Int32 nMode,
                Double dNumber,
                Double dOrderNumber,
                [MarshalAs(UnmanagedType.LPStr)]string ClassCode,
                [MarshalAs(UnmanagedType.LPStr)]string SecCode,
                Double dPrice,
                Int32 nQty,
                Double dValue,
                Int32 nIsSell,
                Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_START_TRADES@4", CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 start_trades(
            trade_status_callback pfTradeStatusCallback);
        #endregion

        #region trade_descriptor_functions
        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_DATE@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_TRADE_DATE(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_SETTLE_DATE@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_TRADE_SETTLE_DATE(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_TIME@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_TRADE_TIME(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_IS_MARGINAL@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_TRADE_IS_MARGINAL(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_ACCRUED_INT@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_ACCRUED_INT(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_YIELD@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_YIELD(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_TS_COMMISSION@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_TS_COMMISSION(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_CLEARING_CENTER_COMMISSION@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_CLEARING_CENTER_COMMISSION(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_EXCHANGE_COMMISSION@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_EXCHANGE_COMMISSION(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_TRADING_SYSTEM_COMMISSION@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_TRADING_SYSTEM_COMMISSION(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_PRICE2@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_PRICE2(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_REPO_RATE@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_REPO_RATE(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_REPO_VALUE@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_REPO_VALUE(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_REPO2_VALUE@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_REPO2_VALUE(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_ACCRUED_INT2@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_ACCRUED_INT2(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_REPO_TERM@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_TRADE_REPO_TERM(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_START_DISCOUNT@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_START_DISCOUNT(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_LOWER_DISCOUNT@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_LOWER_DISCOUNT(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_UPPER_DISCOUNT@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_TRADE_UPPER_DISCOUNT(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_BLOCK_SECURITIES@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_TRADE_BLOCK_SECURITIES(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_PERIOD@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_TRADE_PERIOD(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_DATE_TIME@8",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_TRADE_DATE_TIME(Int32 nTradeDescriptor, Int32 nTimeType);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_FILETIME@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern System.Runtime.InteropServices.ComTypes.FILETIME TRANS2QUIK_TRADE_FILETIME(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_KIND@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_TRADE_KIND(Int32 nTradeDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_CURRENCY@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern IntPtr TRANS2QUIK_TRADE_CURRENCY_IMPL(Int32 nTradeDescriptor);

        public static string TRANS2QUIK_TRADE_CURRENCY(Int32 nTradeDescriptor)
        {
            return Marshal.PtrToStringAnsi(TRANS2QUIK_TRADE_CURRENCY_IMPL(nTradeDescriptor));
        }

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_SETTLE_CURRENCY@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern IntPtr TRANS2QUIK_TRADE_SETTLE_CURRENCY_IMPL(Int32 nTradeDescriptor);

        public static string TRANS2QUIK_TRADE_SETTLE_CURRENCY(Int32 nTradeDescriptor)
        {
            return Marshal.PtrToStringAnsi(TRANS2QUIK_TRADE_SETTLE_CURRENCY_IMPL(nTradeDescriptor));
        }

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_SETTLE_CODE@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern IntPtr TRANS2QUIK_TRADE_SETTLE_CODE_IMPL(Int32 nTradeDescriptor);

        public static string TRANS2QUIK_TRADE_SETTLE_CODE(Int32 nTradeDescriptor)
        {
            return Marshal.PtrToStringAnsi(TRANS2QUIK_TRADE_SETTLE_CODE_IMPL(nTradeDescriptor));
        }

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_ACCOUNT@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern IntPtr TRANS2QUIK_TRADE_ACCOUNT_IMPL(Int32 nTradeDescriptor);

        public static string TRANS2QUIK_TRADE_ACCOUNT(Int32 nTradeDescriptor)
        {
            return Marshal.PtrToStringAnsi(TRANS2QUIK_TRADE_ACCOUNT_IMPL(nTradeDescriptor));
        }

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_BROKERREF@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern IntPtr TRANS2QUIK_TRADE_BROKERREF_IMPL(Int32 nTradeDescriptor);

        public static string TRANS2QUIK_TRADE_BROKERREF(Int32 nTradeDescriptor)
        {
            return Marshal.PtrToStringAnsi(TRANS2QUIK_TRADE_BROKERREF_IMPL(nTradeDescriptor));
        }

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_CLIENT_CODE@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern IntPtr TRANS2QUIK_TRADE_CLIENT_CODE_IMPL(Int32 nTradeDescriptor);

        public static string TRANS2QUIK_TRADE_CLIENT_CODE(Int32 nTradeDescriptor)
        {
            return Marshal.PtrToStringAnsi(TRANS2QUIK_TRADE_CLIENT_CODE_IMPL(nTradeDescriptor));
        }

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_USERID@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern IntPtr TRANS2QUIK_TRADE_USERID_IMPL(Int32 nTradeDescriptor);

        public static string TRANS2QUIK_TRADE_USERID(Int32 nTradeDescriptor)
        {
            return Marshal.PtrToStringAnsi(TRANS2QUIK_TRADE_USERID_IMPL(nTradeDescriptor));
        }

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_FIRMID@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern IntPtr TRANS2QUIK_TRADE_FIRMID_IMPL(Int32 nTradeDescriptor);

        public static string TRANS2QUIK_TRADE_FIRMID(Int32 nTradeDescriptor)
        {
            return Marshal.PtrToStringAnsi(TRANS2QUIK_TRADE_FIRMID_IMPL(nTradeDescriptor));
        }

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_PARTNER_FIRMID@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern IntPtr TRANS2QUIK_TRADE_PARTNER_FIRMID_IMPL(Int32 nTradeDescriptor);

        public static string TRANS2QUIK_TRADE_PARTNER_FIRMID(Int32 nTradeDescriptor)
        {
            return Marshal.PtrToStringAnsi(TRANS2QUIK_TRADE_PARTNER_FIRMID_IMPL(nTradeDescriptor));
        }

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_EXCHANGE_CODE@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern IntPtr TRANS2QUIK_TRADE_EXCHANGE_CODE_IMPL(Int32 nTradeDescriptor);

        public static string TRANS2QUIK_TRADE_EXCHANGE_CODE(Int32 nTradeDescriptor)
        {
            return Marshal.PtrToStringAnsi(TRANS2QUIK_TRADE_EXCHANGE_CODE_IMPL(nTradeDescriptor));
        }

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_TRADE_STATION_ID@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern IntPtr TRANS2QUIK_TRADE_STATION_ID_IMPL(Int32 nTradeDescriptor);

        public static string TRANS2QUIK_TRADE_STATION_ID(Int32 nTradeDescriptor)
        {
            return Marshal.PtrToStringAnsi(TRANS2QUIK_TRADE_STATION_ID_IMPL(nTradeDescriptor));
        }

        #endregion

        #region orders
        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_SUBSCRIBE_ORDERS@8",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 subscribe_orders([MarshalAs(UnmanagedType.LPStr)]string class_code, [MarshalAs(UnmanagedType.LPStr)]string sec_code);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_UNSUBSCRIBE_ORDERS@0",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 unsubscribe_orders();

        public delegate void order_status_callback(
                Int32 nMode,
                Int32 dwTransID,
                Double dNumber,
                [MarshalAs(UnmanagedType.LPStr)]string ClassCode,
                [MarshalAs(UnmanagedType.LPStr)]string SecCode,
                Double dPrice,
                Int32 nBalance,
                Double dValue,
                Int32 nIsSell,
                Int32 nStatus,
                Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_START_ORDERS@4", CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 start_orders(
            order_status_callback pfOrderStatusCallback);

        #region order_descriptor_functions
        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_QTY@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_ORDER_QTY(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_DATE@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_ORDER_DATE(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_TIME@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_ORDER_TIME(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_ACTIVATION_TIME@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_ORDER_ACTIVATION_TIME(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_WITHDRAW_TIME@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_ORDER_WITHDRAW_TIME(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_EXPIRY@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_ORDER_EXPIRY(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_ACCRUED_INT@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_ORDER_ACCRUED_INT(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_YIELD@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern double TRANS2QUIK_ORDER_YIELD(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_UID@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern Int32 TRANS2QUIK_ORDER_UID(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_USERID@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern string TRANS2QUIK_ORDER_USERID(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_ACCOUNT@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern string TRANS2QUIK_ORDER_ACCOUNT(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_BROKERREF@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern string TRANS2QUIK_ORDER_BROKERREF(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_CLIENT_CODE@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern string TRANS2QUIK_ORDER_CLIENT_CODE(Int32 nOrderDescriptor);

        [DllImport("TRANS2QUIK.DLL", EntryPoint = "_TRANS2QUIK_ORDER_FIRMID@4",
            CallingConvention = CallingConvention.StdCall)]
        public static extern string TRANS2QUIK_ORDER_FIRMID(Int32 nOrderDescriptor);
        #endregion

        static int[] ordActArr = new int[20];
        static int ordActArrPos = -1;
        static Object ordlock=new object();
        static string com;
        static int nMd = -1;

        public static void order_status_callback_impl(
                Int32 nMode, Int32 dwTransID, Double dNumber, string ClassCode, string SecCode,
                Double dPrice, Int32 nBalance, Double dValue, Int32 nIsSell, Int32 nStatus, Int32 nOrderDescriptor)
        {
            string nCom = TRANS2QUIK_ORDER_BROKERREF(nOrderDescriptor);
            if (nStatus == 1 && nCom == com && ordActArrPos < 19 && (nMode == 1 || nMode == 2))
            {
                ordActArrPos++;
                ordActArr[ordActArrPos] = (int)dNumber;
                nMd = nMode;
            }
            if (nMode == 2 || ordActArrPos == 19) nMd = 2;
        }

        public static void readOrder(ref long[] oAA, ref int oAAP, string comment)
        {
            lock(ordlock)
            {
                nMd = -1;
                ordActArrPos = -1;
                com = comment;
                int iter = 0;

                order_status_callback order_callback = new order_status_callback(order_status_callback_impl);
                GCHandle gcOrder = GCHandle.Alloc(order_callback);

                subscribe_orders("", "");
                start_orders(order_callback);

                do
                {
                    iter++;
                    System.Threading.Thread.Sleep(100);
                }while (!((nMd==2)||(nMd==-1 && iter>150)));

                unsubscribe_orders();
                oAAP = ordActArrPos;
                ordActArr.CopyTo(oAA, 0);
            }
        }
        #endregion

        public static string ByteToString(byte[] Str)
        {
            string s = "";
            for (int i = 0; i < Str.Length; i++)
            {
                s = s + Str[i].ToString();
            }
            return s;
        }

        public static void dllConnect(string connstr)
        {
            int errorCode = 0;
            string errorMsg = "";
            uint errMsgSize = 0;
            int connResult = 0;
            connResult = TRANS2QUIK_CONNECT(connstr, ref errorCode, errorMsg, errMsgSize);
            if (connResult != 0)
            {
                MessageBox.Show("No connection with QUIK");
            }
        }
        // Добавление ордера через QUIK
        public static void putFutOrder(int transid, string classcode, string stname, char operation, int amount,
            double price, string account, int nthr)
        {
            string transStr;
            
            lock (flock)
            {
                string pricew = System.String.Format("{0:F0}", price);
                string action = "NEW_ORDER";
                string expire = "GTC";
                transStr = "ACTION=" + action + ";" + "ACCOUNT=" + account + ";" + "TRANS_ID=" + transid + ";"
                    + "CLASSCODE=" + classcode + ";" + "SECCODE=" + stname + ";" + "OPERATION=" + operation + ";"
                    + "QUANTITY=" + amount + ";"
                    + "PRICE=" + pricew + ";" + "CLIENT_CODE=" + nthr + ";" + "EXPIRY_DATE=" + expire + ";";

                int errorCode = 0;
                string errorMsg = "";
                uint errMsgSize = 256;
                int result = 0;
                result = TRANS2QUIK_SEND_ASYNC_TRANSACTION(transStr, ref errorCode, errorMsg, errMsgSize);
            }
            //logWrite(transStr);
        }
        //Передвижение ордера через QUIK
        public static void MoveOrder(string classcode, string stname, int buyamount, int sellamount,
        double buyprice, double sellprice, long buyorder, long sellorder, int transid, int mode)
        {
            string price1, price2;
            price1 = System.String.Format("{0:F0}", buyprice);
            price2 = System.String.Format("{0:F0}", sellprice);
            string straction = "MOVE_ORDERS";
            string order1, amount1;
            order1 = System.String.Format("{0:D}", buyorder);
            amount1 = System.String.Format("{0:D}", buyamount);
            string order2, amount2;
            order2 = System.String.Format("{0:D}", sellorder);
            amount2 = System.String.Format("{0:D}", sellamount);
            if (buyorder == 0)
            {
                order1 = System.String.Format("{0:D}", sellorder);
                amount1 = System.String.Format("{0:D}", sellamount);
                price1 = price2;
                order2 = null;
                amount2 = null;
                price2 = null;
            }
            else if (sellorder == 0)
            {
                order2 = null;
                amount2 = null;
                price2 = null;
            }
            string transStr;
            lock (flock)
            {
                int errorCode = 0;
                string errorMsg = "";
                uint errMsgSize = 256;
                int result = 0;
                transStr = ("ACTION=" + straction + ";" + "TRANS_ID=" + transid + ";"
                    + "CLASSCODE=" + classcode + ";" + "SECCODE=" + stname + ";" + "MODE=" + mode + ";"
                    + "FIRST_ORDER_NUMBER=" + order1 + ";" + "FIRST_ORDER_NEW_PRICE=" + price1 + ";"
                    + "FIRST_ORDER_NEW_QUANTITY=" + amount1 + ";" + "SECOND_ORDER_NUMBER=" + order2 + ";"
                    + "SECOND_ORDER_NEW_PRICE=" + price2 + ";" + "SECOND_ORDER_NEW_QUANTITY=" + amount2 + ";");
                result = TRANS2QUIK_SEND_ASYNC_TRANSACTION(transStr, ref errorCode, errorMsg, errMsgSize);
            }
            //logWrite(transStr);
        }
        //Удаление ордера через QUIK
        public static void killFutOrder(int transid, string account, string basecontr, char operation, string futName, long orderKey)
        {
            lock (flock)
            {
                string classcode = "SPBFUT";
                string action = "KILL_ALL_FUTURES_ORDERS";
                string transStr = "";
                if (orderKey == 0)
                {
                    if (operation == 'N')
                    {
                        transStr = "ACTION=" + action + ";" + "ACCOUNT=" + account + ";" + "TRANS_ID=" + transid + ";"
                            + "CLASSCODE=" + classcode + ";" + "BASE_CONTRACT=" + basecontr + ";";
                    }
                    else
                    {
                        transStr = "ACTION=" + action + ";" + "ACCOUNT=" + account + ";" + "TRANS_ID=" + transid + ";"
                            + "OPERATION=" + operation + ";" + "CLASSCODE=" + classcode + ";" + "BASE_CONTRACT=" + basecontr + ";";
                    }
                }
                else
                {
                    transStr = ("ACTION=KILL_ORDER;" + "ACCOUNT=" + account + ";" + "TRANS_ID=" + transid + ";"
                            + "CLASSCODE=" + classcode + ";" + "SECCODE=" + futName + ";" + "ORDER_KEY=" + orderKey + ";");
                }

                int errorCode = 0;
                string errorMsg = "";
                uint errMsgSize = 256;
                int result = 0;
                result = TRANS2QUIK_SEND_ASYNC_TRANSACTION(transStr, ref errorCode, errorMsg, errMsgSize);
            }
        }


        //Получение основных параметров инструментов из хранилищ данных - trdatafut - для торгуемого фьючерса,
        // trdataind - для активов, составляющих индекс
        public static bool DBReadI2(currTradesData trdatafut,currTradesData trdataind,
       string futName, ref int futtime, ref int indtime, ref int firstTime, ref double[] price,
            ref double[] indprice, ref double[] lastPrices, double[] indCodes, ref double[] volfut, ref double[] volfutN,
            string[] indNames, double kConst, ref int keyValfut, ref int keyValind, string dname, ref double dprice,
            ref int[] frB, ref int[] frS)
        {
            bool b = false;
            if (keyValfut == 0)
            {
                int kf = keyValfut;
                int ki = keyValind;
                string nmfut = "";
                string nmind = "";
                int allpr = 0;
                bool futs = false;
                bool doll = false;
                TimeSpan tind = new TimeSpan(0, 0, 0);
                TimeSpan tfut = new TimeSpan(0, 0, 0);

                while (allpr < lastPrices.Length)
                {
                    nmind = trdataind.name[ki];
                    tind = trdataind.time[ki];
                    if (nmind != "")//(nm != futName && nm != dname)//
                    {
                        bool mtc = false;
                        // Текущие цены бумаг, составляющих индекс, вносятся в массив. Определение цен происходит без считывания имен-
                        // основано на различии цен активов
                        for (int i = 0; i < indNames.Length; i++)
                        {

                            if ((lastPrices[i] / trdataind.price[ki] < 1.03 &&
                                lastPrices[i] / trdataind.price[ki] > 0.97))
                            {
                                lastPrices[i] = trdataind.price[ki];
                                mtc = true;
                                break;
                            }

                        }
                        // Если цена считывается первый раз
                        if (!mtc)
                        {
                            for (int i = 0; i < indNames.Length; i++)
                            {
                                if (lastPrices[i] == 0)
                                {
                                    lastPrices[i] = trdataind.price[ki];
                                    break;
                                }
                            }

                        }
                    }
                    allpr = 0;
                    for (int i = 0; i < indNames.Length; i++)
                    {
                        if (lastPrices[i] != 0) allpr++;
                    }
                    keyValind = ki;
                    if (ki<trdataind.pos) ki++;
                }
                // Первое считывание цен торгуемого фьючерса и валютного фьючерса (участвует в расчете индекса) и установка времени
                while (!futs || !doll)
                {
                    nmfut = trdatafut.name[kf];
                    tfut = trdatafut.time[kf];
                    if (nmfut == futName)
                    {
                        price[0] = trdatafut.price[kf];
                        futs = true;
                        firstTime = (int)tfut.TotalSeconds;
                    }
                    if (nmfut == dname)
                    {
                        dprice = trdatafut.price[kf] / 1000;
                        doll = true;
                        firstTime = (int)tfut.TotalSeconds;
                    }
                    keyValfut = kf;
                    if (kf<trdatafut.pos) kf++;
                }
                if (firstTime < (int)tind.TotalSeconds) firstTime = (int)tind.TotalSeconds;
                // Расчет индекса по текущим ценам из массива lastPrices - первое считывание
                for (int i = lastPrices.Length - 1; i >= 0; i--)
                {
                    for (int l = 1; l <= i; l++)
                    {
                        if (lastPrices[l - 1] < lastPrices[l])
                        {
                            double temp = lastPrices[l - 1];
                            lastPrices[l - 1] = lastPrices[l];
                            lastPrices[l] = temp;
                        }
                        if (indCodes[l - 1] > indCodes[l])
                        {
                            double temp = indCodes[l - 1];
                            indCodes[l - 1] = indCodes[l];
                            indCodes[l] = temp;
                        }
                    }
                }
                for (int j = 0; j < indNames.Length; j++)
                {
                    indprice[0] += kConst / dprice * lastPrices[j] * indCodes[j];
                }
            }
            // Считывание текущей цены торгуемого фьючерса и занесение ее в массив price (посекундно) для дальнейших расчетов,
            // массив volfut - объем покупок(посекундно), массив volfutN - объем продаж, frB - число покупок, frS- число продаж
            // далее - занесение значения индекса в массив indprice (посекундно),в данном цикле это происходит только при изменении цены валютного фьючерса.
            if (keyValfut < trdatafut.pos)
            {
                int newTime = 0;
                for (int h = keyValfut + 1; h <= trdatafut.pos; h++)
                {
                    newTime = (int)trdatafut.time[h].TotalSeconds - firstTime;
                    if (newTime < 0) newTime = 0;
                    string stname = trdatafut.name[h];
                    if (stname == futName)
                    {
                        for (int i = futtime + 1; i < newTime; i++)
                        {
                            price[i] = price[futtime];
                        }
                        price[newTime] = trdatafut.price[h];
                        if (trdatafut.operation[h] == "Купля")
                        {
                            volfut[newTime] += trdatafut.vol[h];
                            frB[newTime]++;
                        }
                        else
                        {
                            volfutN[newTime] += trdatafut.vol[h];
                            frS[newTime]++;
                        }
                        futtime = newTime;
                    }
                    else if (stname == dname)
                    {
                        dprice = trdatafut.price[h] / 1000;
                        if (newTime > indtime)
                        {
                            for (int j = indtime + 1; j <= newTime; j++)
                            {
                                indprice[j] = indprice[indtime];
                            }
                        }
                        indtime = newTime;
                        if (indtime < 0) indtime = 0;
                        indprice[indtime] = 0;
                        for (int j = 0; j < indNames.Length; j++)
                        {
                            indprice[indtime] += kConst * lastPrices[j] * indCodes[j];
                        }
                        indprice[indtime] = indprice[indtime] / dprice;
                    }
                }
                keyValfut = trdatafut.pos;
                b = true;
            }
            //занесение значения индекса в массив indprice (посекундно) - здесь считываются цены активов, составляющих индекс (без валютного фьючерса) 
            if (keyValind < trdataind.pos)
            {
                int newTime = 0;
                for (int h = keyValind + 1; h <= trdataind.pos; h++)
                {
                    newTime = (int)trdataind.time[h].TotalSeconds - firstTime;
                    if (newTime < 0) newTime = 0;
                    string stname = trdataind.name[h];
                    if (stname == "")
                    {
                        double priceCheck = 0;
                        for (int i = 0; i < indNames.Length; i++)
                        {
                            priceCheck = lastPrices[i] / trdataind.price[h];
                            if (priceCheck < 1.03 && priceCheck > 0.97)
                            {
                                if (newTime > indtime)
                                {
                                    for (int j = indtime + 1; j <= newTime; j++)
                                    {
                                        indprice[j] = indprice[indtime];
                                    }
                                }
                                lastPrices[i] = trdataind.price[h];

                                indtime = newTime;
                                if (indtime < 0) indtime = 0;
                                indprice[indtime] = 0;
                                for (int j = 0; j < indNames.Length; j++)
                                {
                                    indprice[indtime] += kConst * lastPrices[j] * indCodes[j];
                                }
                                indprice[indtime] = indprice[indtime] / dprice;
                            }
                        }
                    }
                }
                keyValind = trdataind.pos;
                b = true;
            }
            return b;
        }

        //Получение данных по своим сделкам из хранилища exec
        public static void DBReadExec(execord exec,string futname, ref int pozbuy, ref int pozsell, ref double summ, 
            ref int code, ref int contr,ref int trd, int nthr)
        {
           
            if (code<exec.pos)
            {
                for (int i = code + 1; i <= exec.pos;i++)
                {
                    if (exec.exename[i] == futname && exec.comment[i]==nthr)
                    {
                        if (exec.exedir[i] == "Купля")
                        {
                            pozbuy += exec.exequan[i];
                            summ -= exec.exequan[i] * exec.exeprice[i];
                        }

                        else if (exec.exedir[i] == "Продажа")
                        {
                            pozsell += exec.exequan[i];
                            summ += exec.exequan[i] * exec.exeprice[i];
                        }
                        contr += exec.exequan[i];
                        trd++;
                    }
                }
                code = exec.pos;
            }
        }

        //Функция простого суммирования
        public static int Summ(int intsec, int N, int[] price)
        {
            int spr;
            int sum = 0;
            if (N - intsec < 0)
                spr = N;
            else
                spr = intsec;
            for (int i = N - spr; i <= N; ++i)
            {
                sum += price[i];
            }
            return sum;

        }

        //Функция нахождения среднего арифметического
        public static double timeInt(int intsec, int N, double[] price)
        {
            int spr;
            double sma = 0, sum = 0;
            if (N - intsec < 0)
                spr = N;
            else
                spr = intsec;
            for (int i = N - spr; i <= N; ++i)
            {
                sum += price[i];
            }
            sma = sum / (double)(spr+1);
            return sma;
        }

        //Функция нахождения среднеквадратического отклонения
        public static double timeIntS(int intsec, double[] spread, int N)
        {
            int spr;
            if (N - intsec < 0)
                spr = N;
            else
                spr = intsec;
            double sumq = 0, sigma = 0;
            double avg = 0;
            for (int i = N - spr; i <= N; ++i)
            {
                sumq += (spread[i]);
            }
            avg = sumq /(double)(spr + 1);
            sumq = 0;
            for (int i = N - spr; i <= N; ++i)
            {
                sumq += (spread[i] - avg) * (spread[i] - avg);
            }
            if (spr == 0)
                sigma = 0;
            else
                sigma = Math.Sqrt(sumq /(double)(spr+1));
            return sigma;
        }

        //Функция нахождения среднего арифметического
        public static double Average(double[] d)
        {
            int N;
            double summ = 0, av = 0;
            N = d.Length - 1;
            for (int i = 0; i <= N; ++i)
            {
                summ += d[i];
            }
            av = summ / (double)(N + 1);
            return av;
        }

        //Функция записи результатов в файл
        public static void resWrite(string mes)
        {
            lock (loglock)
            {
                System.IO.StreamWriter rwrite = new System.IO.StreamWriter(@"result.txt", true);
                rwrite.WriteLine(mes);
                rwrite.Close();
            }
        }
    }
}
