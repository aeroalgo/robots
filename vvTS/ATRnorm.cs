using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200000C RID: 12
	[HandlerCategory("vvIndicators"), HandlerName("ATR normalized")]
	public class ATRnorm : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000060 RID: 96 RVA: 0x00003424 File Offset: 0x00001624
		public IList<double> Execute(ISecurity sec)
		{
			return this.GenATRnrm(sec, this.AtrPeriod, this.NormPeriod);
		}

		// Token: 0x0600005F RID: 95 RVA: 0x000032A0 File Offset: 0x000014A0
		public IList<double> GenATRnrm(ISecurity src, int atrperiod, int normperiod)
		{
			int count = src.get_Bars().Count;
			int num = Math.Max(this.AtrPeriod, this.NormPeriod);
			double[] array = new double[count];
			double[] array2 = new double[count];
			for (int i = 0; i < num; i++)
			{
				array[i] = 0.0;
				array2[i] = 0.0;
			}
			for (int j = 0; j < count; j++)
			{
				double num2 = src.get_HighPrices()[j];
				double num3 = src.get_LowPrices()[j];
				if (j == 0)
				{
					array2[j] = num2 - num3;
				}
				else
				{
					double val = src.get_ClosePrices()[j - 1];
					array2[j] = Math.Max(num2, val) - Math.Min(num3, val);
				}
			}
			IList<double> list = SMA.GenSMA(array2, this.AtrPeriod);
			for (int k = num; k < count; k++)
			{
				double num4 = 1E-07;
				double num5 = 1000000.0;
				for (int l = 0; l < this.NormPeriod; l++)
				{
					num4 = Math.Max(num4, list[k - l]);
					num5 = Math.Min(num5, list[k - l]);
				}
				if (num4 - num5 > 0.0)
				{
					array[k] = 100.0 * (list[k] - num5) / (num4 - num5);
				}
			}
			return array;
		}

		// Token: 0x1700001C RID: 28
		[HandlerParameter(true, "5", Min = "1", Max = "100", Step = "1")]
		public int AtrPeriod
		{
			// Token: 0x0600005B RID: 91 RVA: 0x0000327D File Offset: 0x0000147D
			get;
			// Token: 0x0600005C RID: 92 RVA: 0x00003285 File Offset: 0x00001485
			set;
		}

		// Token: 0x1700001D RID: 29
		[HandlerParameter(true, "100", Min = "0", Max = "100", Step = "5")]
		public int NormPeriod
		{
			// Token: 0x0600005D RID: 93 RVA: 0x0000328E File Offset: 0x0000148E
			get;
			// Token: 0x0600005E RID: 94 RVA: 0x00003296 File Offset: 0x00001496
			set;
		}
	}
}
