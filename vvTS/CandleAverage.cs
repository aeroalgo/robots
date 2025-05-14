using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200015B RID: 347
	[HandlerCategory("vvAverages"), HandlerName("Candle Average")]
	public class CandleAverage : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000AEF RID: 2799 RVA: 0x0002CF44 File Offset: 0x0002B144
		public IList<double> Execute(ISecurity src)
		{
			int count = src.get_Bars().Count;
			IList<double> arg_21_0 = src.get_ClosePrices();
			IList<double> high = src.get_HighPrices();
			IList<double> arg_35_0 = src.get_LowPrices();
			double[] array = new double[count];
			IList<double> data = this.Context.GetData("H_arr", new string[]
			{
				this.H_period.ToString(),
				src.GetHashCode().ToString()
			}, () => EMA.GenEMA(high, this.H_period));
			IList<double> data2 = this.Context.GetData("L_arr", new string[]
			{
				this.L_period.ToString(),
				src.GetHashCode().ToString()
			}, () => EMA.GenEMA(high, this.L_period));
			IList<double> data3 = this.Context.GetData("C_arr", new string[]
			{
				this.C_period.ToString(),
				src.GetHashCode().ToString()
			}, () => EMA.GenEMA(high, this.C_period));
			for (int i = 0; i < count; i++)
			{
				double num = data[i] - data3[i];
				double num2 = data3[i] - data2[i];
				if (num < num2)
				{
					array[i] = 1.0;
				}
				if (num > num2)
				{
					array[i] = -1.0;
				}
				if (num == num2)
				{
					array[i] = 0.0;
				}
			}
			return SMA.GenSMA(array, this.Length);
		}

		// Token: 0x170003A0 RID: 928
		public IContext Context
		{
			// Token: 0x06000AF0 RID: 2800 RVA: 0x0002D0F6 File Offset: 0x0002B2F6
			get;
			// Token: 0x06000AF1 RID: 2801 RVA: 0x0002D0FE File Offset: 0x0002B2FE
			set;
		}

		// Token: 0x1700039F RID: 927
		[HandlerParameter(true, "9", Min = "5", Max = "50", Step = "1")]
		public int C_period
		{
			// Token: 0x06000AED RID: 2797 RVA: 0x0002CEE1 File Offset: 0x0002B0E1
			get;
			// Token: 0x06000AEE RID: 2798 RVA: 0x0002CEE9 File Offset: 0x0002B0E9
			set;
		}

		// Token: 0x1700039D RID: 925
		[HandlerParameter(true, "25", Min = "5", Max = "50", Step = "1")]
		public int H_period
		{
			// Token: 0x06000AE9 RID: 2793 RVA: 0x0002CEBF File Offset: 0x0002B0BF
			get;
			// Token: 0x06000AEA RID: 2794 RVA: 0x0002CEC7 File Offset: 0x0002B0C7
			set;
		}

		// Token: 0x1700039C RID: 924
		[HandlerParameter(true, "31", Min = "5", Max = "50", Step = "1")]
		public int Length
		{
			// Token: 0x06000AE7 RID: 2791 RVA: 0x0002CEAE File Offset: 0x0002B0AE
			get;
			// Token: 0x06000AE8 RID: 2792 RVA: 0x0002CEB6 File Offset: 0x0002B0B6
			set;
		}

		// Token: 0x1700039E RID: 926
		[HandlerParameter(true, "27", Min = "5", Max = "50", Step = "1")]
		public int L_period
		{
			// Token: 0x06000AEB RID: 2795 RVA: 0x0002CED0 File Offset: 0x0002B0D0
			get;
			// Token: 0x06000AEC RID: 2796 RVA: 0x0002CED8 File Offset: 0x0002B0D8
			set;
		}
	}
}
