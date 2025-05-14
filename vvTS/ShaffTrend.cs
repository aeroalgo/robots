using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000058 RID: 88
	[HandlerCategory("vvIndicators"), HandlerName("Shaff Trend Cycle")]
	public class ShaffTrend : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000326 RID: 806 RVA: 0x00012164 File Offset: 0x00010364
		public IList<double> Execute(IList<double> src)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			double[] array6 = new double[count];
			double num = 2.0 / (1.0 + (double)this.CDPeriod);
			double num2 = src[0];
			double num3 = src[0];
			int sTCPeriod = this.STCPeriod;
			for (int i = sTCPeriod; i < count; i++)
			{
				num2 = EMA.iEMA(src[i], num2, this.FastMAPeriod, i);
				num3 = EMA.iEMA(src[i], num3, this.SlowMAPeriod, i);
				array2[i] = num2 - num3;
				array3[i] = array3[i - 1] + num * (array2[i] - array3[i - 1]);
				double num4 = this.minValue(array3, i, this.STCPeriod);
				double num5 = this.maxValue(array3, i, this.STCPeriod) - num4;
				if (num5 > 0.0)
				{
					array4[i] = 100.0 * ((array3[i] - num4) / num5);
				}
				else
				{
					array4[i] = array4[i - 1];
				}
				array5[i] = array5[i - 1] + 0.5 * (array4[i] - array5[i - 1]);
				double num6 = this.minValue(array5, i, this.STCPeriod);
				double num7 = this.maxValue(array5, i, this.STCPeriod) - num6;
				if (num7 > 0.0)
				{
					array6[i] = 100.0 * ((array5[i] - num6) / num7);
				}
				else
				{
					array6[i] = array6[i - 1];
				}
				array[i] = array[i - 1] + 0.5 * (array6[i] - array[i - 1]);
			}
			return array;
		}

		// Token: 0x06000328 RID: 808 RVA: 0x00012380 File Offset: 0x00010580
		private double maxValue(IList<double> array, int shift, int period)
		{
			double num = array[shift];
			for (int i = 1; i < period; i++)
			{
				num = Math.Max(num, array[shift - i]);
			}
			return num;
		}

		// Token: 0x06000327 RID: 807 RVA: 0x0001234C File Offset: 0x0001054C
		private double minValue(IList<double> array, int shift, int period)
		{
			double num = array[shift];
			for (int i = 1; i < period; i++)
			{
				num = Math.Min(num, array[shift - i]);
			}
			return num;
		}

		// Token: 0x17000110 RID: 272
		[HandlerParameter(true, "1", Min = "1", Max = "25", Step = "1")]
		public int CDPeriod
		{
			// Token: 0x06000324 RID: 804 RVA: 0x00012152 File Offset: 0x00010352
			get;
			// Token: 0x06000325 RID: 805 RVA: 0x0001215A File Offset: 0x0001035A
			set;
		}

		// Token: 0x17000111 RID: 273
		public IContext Context
		{
			// Token: 0x06000329 RID: 809 RVA: 0x000123B2 File Offset: 0x000105B2
			get;
			// Token: 0x0600032A RID: 810 RVA: 0x000123BA File Offset: 0x000105BA
			set;
		}

		// Token: 0x1700010D RID: 269
		[HandlerParameter(true, "23", Min = "5", Max = "100", Step = "1")]
		public int FastMAPeriod
		{
			// Token: 0x0600031E RID: 798 RVA: 0x0001211F File Offset: 0x0001031F
			get;
			// Token: 0x0600031F RID: 799 RVA: 0x00012127 File Offset: 0x00010327
			set;
		}

		// Token: 0x1700010E RID: 270
		[HandlerParameter(true, "50", Min = "20", Max = "200", Step = "1")]
		public int SlowMAPeriod
		{
			// Token: 0x06000320 RID: 800 RVA: 0x00012130 File Offset: 0x00010330
			get;
			// Token: 0x06000321 RID: 801 RVA: 0x00012138 File Offset: 0x00010338
			set;
		}

		// Token: 0x1700010F RID: 271
		[HandlerParameter(true, "10", Min = "5", Max = "30", Step = "1")]
		public int STCPeriod
		{
			// Token: 0x06000322 RID: 802 RVA: 0x00012141 File Offset: 0x00010341
			get;
			// Token: 0x06000323 RID: 803 RVA: 0x00012149 File Offset: 0x00010349
			set;
		}
	}
}
