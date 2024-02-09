using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000017 RID: 23
	[HandlerCategory("vvIndicators"), HandlerName("Chaikin's Oscillator")]
	public class ChaikinOscillator : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060000BC RID: 188 RVA: 0x00004448 File Offset: 0x00002648
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> volumes = sec.get_Volumes();
			IList<double> VA = new List<double>(closePrices.Count);
			IList<double> list = new List<double>(closePrices.Count);
			for (int i = 0; i < closePrices.Count; i++)
			{
				double item;
				if (highPrices[i] != lowPrices[i])
				{
					item = (closePrices[i] - lowPrices[i] - (highPrices[i] - closePrices[i])) / (highPrices[i] - lowPrices[i]) * volumes[i];
				}
				else
				{
					item = (closePrices[i] - lowPrices[i] - (highPrices[i] - closePrices[i])) / (highPrices[i] - lowPrices[i] + 1E-08) * volumes[i];
				}
				VA.Add(item);
			}
			IList<double> data = this.Context.GetData("sma", new string[]
			{
				this.PeriodM.ToString(),
				VA.GetHashCode().ToString()
			}, () => Series.SMA(VA, this.PeriodM));
			IList<double> data2 = this.Context.GetData("sma", new string[]
			{
				this.PeriodN.ToString(),
				VA.GetHashCode().ToString()
			}, () => Series.SMA(VA, this.PeriodN));
			for (int j = 0; j < closePrices.Count; j++)
			{
				double item2 = data[j] - data2[j];
				list.Add(item2);
			}
			return list;
		}

		// Token: 0x1700003D RID: 61
		public IContext Context
		{
			// Token: 0x060000BD RID: 189 RVA: 0x00004645 File Offset: 0x00002845
			get;
			// Token: 0x060000BE RID: 190 RVA: 0x0000464D File Offset: 0x0000284D
			set;
		}

		// Token: 0x1700003B RID: 59
		[HandlerParameter(true, "20", Min = "5", Max = "20", Step = "1")]
		public int PeriodM
		{
			// Token: 0x060000B8 RID: 184 RVA: 0x000043EB File Offset: 0x000025EB
			get;
			// Token: 0x060000B9 RID: 185 RVA: 0x000043F3 File Offset: 0x000025F3
			set;
		}

		// Token: 0x1700003C RID: 60
		[HandlerParameter(true, "10", Min = "5", Max = "20", Step = "1")]
		public int PeriodN
		{
			// Token: 0x060000BA RID: 186 RVA: 0x000043FC File Offset: 0x000025FC
			get;
			// Token: 0x060000BB RID: 187 RVA: 0x00004404 File Offset: 0x00002604
			set;
		}
	}
}
