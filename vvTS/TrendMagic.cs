using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000064 RID: 100
	[HandlerCategory("vvIndicators"), HandlerName("TrendMagic")]
	public class TrendMagic : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000382 RID: 898 RVA: 0x00013B18 File Offset: 0x00011D18
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> data = this.Context.GetData("atr", new string[]
			{
				this.PeriodATR.ToString(),
				sec.get_CacheName()
			}, () => Series.AverageTrueRange(sec.get_Bars(), this.PeriodATR));
			IList<double> data2 = this.Context.GetData("cci", new string[]
			{
				this.PeriodCCI.ToString(),
				sec.get_CacheName()
			}, () => Series.CCI(sec.get_Bars(), this.PeriodCCI));
			IList<double> list = new List<double>(closePrices.Count);
			for (int i = 0; i < closePrices.Count; i++)
			{
				if (i == 0)
				{
					list.Add((highPrices[i] + lowPrices[i]) / 2.0);
				}
				else
				{
					double num;
					if (data2[i] >= 0.0)
					{
						num = lowPrices[i] - data[i];
						num = ((num < list[i - 1]) ? list[i - 1] : num);
					}
					else
					{
						num = highPrices[i] + data[i];
						num = ((num > list[i - 1]) ? list[i - 1] : num);
					}
					list.Add(num);
				}
			}
			return list;
		}

		// Token: 0x1700012D RID: 301
		public IContext Context
		{
			// Token: 0x06000383 RID: 899 RVA: 0x00013CCB File Offset: 0x00011ECB
			get;
			// Token: 0x06000384 RID: 900 RVA: 0x00013CD3 File Offset: 0x00011ED3
			set;
		}

		// Token: 0x1700012C RID: 300
		[HandlerParameter(true, "5", Min = "1", Max = "30", Step = "1")]
		public int PeriodATR
		{
			// Token: 0x06000380 RID: 896 RVA: 0x00013AC5 File Offset: 0x00011CC5
			get;
			// Token: 0x06000381 RID: 897 RVA: 0x00013ACD File Offset: 0x00011CCD
			set;
		}

		// Token: 0x1700012B RID: 299
		[HandlerParameter(true, "50", Min = "5", Max = "100", Step = "1")]
		public int PeriodCCI
		{
			// Token: 0x0600037E RID: 894 RVA: 0x00013AB4 File Offset: 0x00011CB4
			get;
			// Token: 0x0600037F RID: 895 RVA: 0x00013ABC File Offset: 0x00011CBC
			set;
		}
	}
}
