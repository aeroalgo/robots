using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000037 RID: 55
	[HandlerCategory("vvIndicators"), HandlerName("Market Facilitation Index")]
	public class MFI_Williams_max : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060001FA RID: 506 RVA: 0x000097FC File Offset: 0x000079FC
		public IList<double> Execute(ISecurity sec)
		{
			return MFI_Williams_max.GenMFI(sec, this.Context, this.Period);
		}

		// Token: 0x060001F9 RID: 505 RVA: 0x00009668 File Offset: 0x00007868
		public static IList<double> GenMFI(ISecurity sec, IContext Context, int _Period)
		{
			IList<double> data = Context.GetData("hhv", new string[]
			{
				_Period.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(sec.get_HighPrices(), _Period));
			IList<double> data2 = Context.GetData("llv", new string[]
			{
				_Period.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(sec.get_LowPrices(), _Period));
			IList<double> data3 = Context.GetData("HighestVolume", new string[]
			{
				_Period.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(sec.get_Volumes(), _Period));
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> list = new List<double>(closePrices.Count);
			for (int i = 0; i < closePrices.Count; i++)
			{
				double item;
				if (data[i] - data2[i] != 0.0)
				{
					item = (data[i] - data2[i]) / data3[i];
				}
				else
				{
					item = (data[i] - data2[i] + 1E-10) / data3[i];
				}
				list.Add(item);
			}
			return list;
		}

		// Token: 0x170000AB RID: 171
		public IContext Context
		{
			// Token: 0x060001FB RID: 507 RVA: 0x0000981D File Offset: 0x00007A1D
			get;
			// Token: 0x060001FC RID: 508 RVA: 0x00009825 File Offset: 0x00007A25
			set;
		}

		// Token: 0x170000AA RID: 170
		[HandlerParameter]
		public int Period
		{
			// Token: 0x060001F7 RID: 503 RVA: 0x00009605 File Offset: 0x00007805
			get;
			// Token: 0x060001F8 RID: 504 RVA: 0x0000960D File Offset: 0x0000780D
			set;
		}
	}
}
