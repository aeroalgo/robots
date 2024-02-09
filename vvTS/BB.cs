using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000118 RID: 280
	[HandlerCategory("vvBands&Channels"), HandlerName("Bollinger Bands [mod]")]
	public class BB : IDoubleAccumHandler, ITwoSourcesHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x060007DC RID: 2012 RVA: 0x00021E98 File Offset: 0x00020098
		public IList<double> Execute(IList<double> src1, IList<double> src2)
		{
			double[] array = new double[src1.Count];
			for (int i = 1; i < src1.Count; i++)
			{
				double num = Indicators.StDev(src1, src2, i, this.Period);
				array[i] = src2[i] + (double)((this.TopBand ? 1 : -1) * this.Coef) * num;
			}
			return array;
		}

		// Token: 0x17000279 RID: 633
		[HandlerParameter(true, "2", Min = "0", Max = "20", Step = "1")]
		public int Coef
		{
			// Token: 0x060007D8 RID: 2008 RVA: 0x00021E74 File Offset: 0x00020074
			get;
			// Token: 0x060007D9 RID: 2009 RVA: 0x00021E7C File Offset: 0x0002007C
			set;
		}

		// Token: 0x17000278 RID: 632
		[HandlerParameter(true, "20", Min = "0", Max = "50", Step = "1")]
		public int Period
		{
			// Token: 0x060007D6 RID: 2006 RVA: 0x00021E63 File Offset: 0x00020063
			get;
			// Token: 0x060007D7 RID: 2007 RVA: 0x00021E6B File Offset: 0x0002006B
			set;
		}

		// Token: 0x1700027A RID: 634
		[HandlerParameter(false, "true", NotOptimized = true)]
		public bool TopBand
		{
			// Token: 0x060007DA RID: 2010 RVA: 0x00021E85 File Offset: 0x00020085
			get;
			// Token: 0x060007DB RID: 2011 RVA: 0x00021E8D File Offset: 0x0002008D
			set;
		}
	}
}
