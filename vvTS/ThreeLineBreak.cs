using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200005E RID: 94
	[HandlerCategory("vvIndicators"), HandlerName("ThreeLineBreak")]
	public class ThreeLineBreak : IBar2BarHandler, IOneSourceHandler, IStreamHandler, IHandler, ISecurityReturns, ISecurityInputs, IContextUses
	{
		// Token: 0x0600035B RID: 859 RVA: 0x00013224 File Offset: 0x00011424
		public ISecurity Execute(ISecurity sec)
		{
			int count = sec.get_Bars().Count;
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> High = sec.get_HighPrices();
			IList<double> Low = sec.get_LowPrices();
			List<Bar> list = new List<Bar>(count);
			IList<double> data = this.Context.GetData("llv", new string[]
			{
				this.LinesBreak.ToString(),
				sec.get_CacheName()
			}, () => vvSeries.Lowest(Low, this.LinesBreak));
			IList<double> data2 = this.Context.GetData("hhv", new string[]
			{
				this.LinesBreak.ToString(),
				sec.get_CacheName()
			}, () => vvSeries.Highest(High, this.LinesBreak));
			bool flag = true;
			for (int i = 0; i < count; i++)
			{
				Bar bar = sec.get_Bars()[i];
				if (i >= this.LinesBreak)
				{
					if (closePrices[i] < data[i - 1])
					{
						flag = false;
					}
					if (closePrices[i] > data2[i - 1])
					{
						flag = true;
					}
				}
				Bar bar2 = new Bar(bar.get_Color(), bar.get_Date(), flag ? Low[i] : High[i], High[i], Low[i], flag ? High[i] : Low[i], bar.get_Volume());
				bar2.set_Ask(bar.get_Ask());
				bar2.set_Bid(bar.get_Bid());
				bar2.set_AskQty(bar.get_AskQty());
				bar2.set_BidQty(bar.get_BidQty());
				Bar item = bar2;
				list.Add(item);
			}
			return sec.CloneAndReplaceBars(list);
		}

		// Token: 0x17000121 RID: 289
		public IContext Context
		{
			// Token: 0x0600035C RID: 860 RVA: 0x00013427 File Offset: 0x00011627
			get;
			// Token: 0x0600035D RID: 861 RVA: 0x0001342F File Offset: 0x0001162F
			set;
		}

		// Token: 0x17000120 RID: 288
		[HandlerParameter(true, "3", Min = "3", Max = "6", Step = "1")]
		public int LinesBreak
		{
			// Token: 0x06000359 RID: 857 RVA: 0x000131DA File Offset: 0x000113DA
			get;
			// Token: 0x0600035A RID: 858 RVA: 0x000131E2 File Offset: 0x000113E2
			set;
		}
	}
}
