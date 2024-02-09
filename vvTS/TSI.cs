using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000062 RID: 98
	[HandlerCategory("vvIndicators"), HandlerName("Trend Strength Index")]
	public class TSI : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000374 RID: 884 RVA: 0x00013850 File Offset: 0x00011A50
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> data = this.Context.GetData("hhv", new string[]
			{
				this.IntervalSize.ToString(),
				sec.get_HighPrices().GetHashCode().ToString()
			}, () => vvSeries.Highest(sec.get_HighPrices(), this.IntervalSize));
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> list = new List<double>(closePrices.Count);
			for (int i = 0; i < closePrices.Count; i++)
			{
				double num = 0.0;
				int num2 = 0;
				double item = sec.get_HighPrices()[i];
				for (int j = 0; j < this.IntervalCount; j++)
				{
					int num3 = i - j * this.IntervalSize;
					if (num3 >= 0)
					{
						num += data[num3];
						num2++;
					}
				}
				if (num2 > 0)
				{
					item = num / (double)num2;
				}
				list.Add(item);
			}
			return list;
		}

		// Token: 0x17000128 RID: 296
		public IContext Context
		{
			// Token: 0x06000375 RID: 885 RVA: 0x00013969 File Offset: 0x00011B69
			get;
			// Token: 0x06000376 RID: 886 RVA: 0x00013971 File Offset: 0x00011B71
			set;
		}

		// Token: 0x17000127 RID: 295
		[HandlerParameter(true, "10", Min = "2", Max = "20", Step = "2")]
		public int IntervalCount
		{
			// Token: 0x06000372 RID: 882 RVA: 0x00013818 File Offset: 0x00011A18
			get;
			// Token: 0x06000373 RID: 883 RVA: 0x00013820 File Offset: 0x00011A20
			set;
		}

		// Token: 0x17000126 RID: 294
		[HandlerParameter(true, "6", Min = "2", Max = "20", Step = "1")]
		public int IntervalSize
		{
			// Token: 0x06000370 RID: 880 RVA: 0x00013807 File Offset: 0x00011A07
			get;
			// Token: 0x06000371 RID: 881 RVA: 0x0001380F File Offset: 0x00011A0F
			set;
		}
	}
}
