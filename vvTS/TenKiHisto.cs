using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000085 RID: 133
	[HandlerCategory("vvIchimoku"), HandlerName("TenkanKijun Histo")]
	public class TenKiHisto : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600049B RID: 1179 RVA: 0x00017A0C File Offset: 0x00015C0C
		public IList<double> Execute(ISecurity _sec)
		{
			IList<double> high = _sec.get_HighPrices();
			IList<double> low = _sec.get_LowPrices();
			IList<double> arg_2F_0 = _sec.get_ClosePrices();
			double[] array = new double[high.Count];
			IList<double> data = this.Context.GetData("hhv", new string[]
			{
				this.Tenkan.ToString(),
				_sec.get_CacheName()
			}, () => Series.Highest(high, this.Tenkan));
			IList<double> data2 = this.Context.GetData("llv", new string[]
			{
				this.Tenkan.ToString(),
				_sec.get_CacheName()
			}, () => Series.Lowest(low, this.Tenkan));
			IList<double> data3 = this.Context.GetData("hhv", new string[]
			{
				this.Kijun.ToString(),
				_sec.get_CacheName()
			}, () => Series.Highest(high, this.Kijun));
			IList<double> data4 = this.Context.GetData("llv", new string[]
			{
				this.Kijun.ToString(),
				_sec.get_CacheName()
			}, () => Series.Lowest(low, this.Kijun));
			double[] array2 = new double[high.Count];
			double[] array3 = new double[high.Count];
			for (int i = 0; i < high.Count; i++)
			{
				array2[i] = 0.5 * (data[i] + data2[i]);
				array3[i] = 0.5 * (data3[i] + data4[i]);
				array[i] = array2[i] - array3[i];
			}
			return array;
		}

		// Token: 0x17000191 RID: 401
		public IContext Context
		{
			// Token: 0x0600049C RID: 1180 RVA: 0x00017C02 File Offset: 0x00015E02
			get;
			// Token: 0x0600049D RID: 1181 RVA: 0x00017C0A File Offset: 0x00015E0A
			set;
		}

		// Token: 0x17000190 RID: 400
		[HandlerParameter(true, "20", Min = "10", Max = "50", Step = "1")]
		public int Kijun
		{
			// Token: 0x06000499 RID: 1177 RVA: 0x00017991 File Offset: 0x00015B91
			get;
			// Token: 0x0600049A RID: 1178 RVA: 0x00017999 File Offset: 0x00015B99
			set;
		}

		// Token: 0x1700018F RID: 399
		[HandlerParameter(true, "10", Min = "5", Max = "15", Step = "1")]
		public int Tenkan
		{
			// Token: 0x06000497 RID: 1175 RVA: 0x00017980 File Offset: 0x00015B80
			get;
			// Token: 0x06000498 RID: 1176 RVA: 0x00017988 File Offset: 0x00015B88
			set;
		}
	}
}
