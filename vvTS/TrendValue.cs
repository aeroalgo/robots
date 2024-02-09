using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000065 RID: 101
	[HandlerCategory("vvIndicators"), HandlerName("Trend Value")]
	public class TrendValue : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600038E RID: 910 RVA: 0x00013D80 File Offset: 0x00011F80
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> High = sec.get_HighPrices();
			IList<double> Low = sec.get_LowPrices();
			int count = sec.get_Bars().Count;
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> data = this.Context.GetData("atr", new string[]
			{
				this.atrPeriod.ToString(),
				sec.get_CacheName()
			}, () => Series.AverageTrueRange(sec.get_Bars(), this.atrPeriod));
			IList<double> data2 = this.Context.GetData("lwma", new string[]
			{
				this.Period.ToString(),
				High.GetHashCode().ToString()
			}, () => LWMA.GenWMA(High, this.Period));
			IList<double> data3 = this.Context.GetData("lwma", new string[]
			{
				this.Period.ToString(),
				Low.GetHashCode().ToString()
			}, () => LWMA.GenWMA(Low, this.Period));
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			double[] array6 = new double[count];
			for (int i = 1; i < count; i++)
			{
				double num = data[i] * this.atrSensitivity;
				array2[i] = data2[i] * (1.0 + this.shiftPercent / 100.0) + num * this.atrSensitivity;
				array3[i] = data3[i] * (1.0 - this.shiftPercent / 100.0) - num * this.atrSensitivity;
				array4[i] = array4[i - 1];
				if (closePrices[i] > array2[i - 1])
				{
					array4[i] = 1.0;
				}
				if (closePrices[i] < array3[i - 1])
				{
					array4[i] = -1.0;
				}
				if (array4[i] > 0.0)
				{
					if (array3[i] < array3[i - 1])
					{
						array3[i] = array3[i - 1];
					}
					array5[i] = array3[i];
					array6[i] = 0.0;
				}
				else
				{
					if (array2[i] > array2[i - 1])
					{
						array2[i] = array2[i - 1];
					}
					array6[i] = array2[i];
					array5[i] = 0.0;
				}
				array[i] = ((array4[i] > 0.0) ? array3[i] : array2[i]);
			}
			return array;
		}

		// Token: 0x17000130 RID: 304
		[HandlerParameter(true, "10", Min = "1", Max = "30", Step = "1")]
		public int atrPeriod
		{
			// Token: 0x0600038A RID: 906 RVA: 0x00013D06 File Offset: 0x00011F06
			get;
			// Token: 0x0600038B RID: 907 RVA: 0x00013D0E File Offset: 0x00011F0E
			set;
		}

		// Token: 0x17000131 RID: 305
		[HandlerParameter(true, "1.5", Min = "0.1", Max = "3", Step = "0.1")]
		public double atrSensitivity
		{
			// Token: 0x0600038C RID: 908 RVA: 0x00013D17 File Offset: 0x00011F17
			get;
			// Token: 0x0600038D RID: 909 RVA: 0x00013D1F File Offset: 0x00011F1F
			set;
		}

		// Token: 0x17000132 RID: 306
		public IContext Context
		{
			// Token: 0x0600038F RID: 911 RVA: 0x0001407C File Offset: 0x0001227C
			get;
			// Token: 0x06000390 RID: 912 RVA: 0x00014084 File Offset: 0x00012284
			set;
		}

		// Token: 0x1700012E RID: 302
		[HandlerParameter(true, "13", Min = "5", Max = "40", Step = "1")]
		public int Period
		{
			// Token: 0x06000386 RID: 902 RVA: 0x00013CE4 File Offset: 0x00011EE4
			get;
			// Token: 0x06000387 RID: 903 RVA: 0x00013CEC File Offset: 0x00011EEC
			set;
		}

		// Token: 0x1700012F RID: 303
		[HandlerParameter(true, "0", Min = "0", Max = "5", Step = "0.1")]
		public double shiftPercent
		{
			// Token: 0x06000388 RID: 904 RVA: 0x00013CF5 File Offset: 0x00011EF5
			get;
			// Token: 0x06000389 RID: 905 RVA: 0x00013CFD File Offset: 0x00011EFD
			set;
		}
	}
}
