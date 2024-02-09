using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000117 RID: 279
	[HandlerCategory("vvBands&Channels"), HandlerName("Bollinger Bands [std]")]
	public class BollingerBands : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x060007D4 RID: 2004 RVA: 0x00021E3B File Offset: 0x0002003B
		public IList<double> Execute(IList<double> src)
		{
			return Series.BollingerBands(src, this.Period, this.Coef, this.TopBand);
		}

		// Token: 0x17000277 RID: 631
		[HandlerParameter(true, "2", Min = "0.5", Max = "3", Step = "0.5")]
		public double Coef
		{
			// Token: 0x060007D2 RID: 2002 RVA: 0x00021E2A File Offset: 0x0002002A
			get;
			// Token: 0x060007D3 RID: 2003 RVA: 0x00021E32 File Offset: 0x00020032
			set;
		}

		// Token: 0x17000275 RID: 629
		[HandlerParameter(true, "20", Min = "0", Max = "50", Step = "1")]
		public int Period
		{
			// Token: 0x060007CE RID: 1998 RVA: 0x00021E08 File Offset: 0x00020008
			get;
			// Token: 0x060007CF RID: 1999 RVA: 0x00021E10 File Offset: 0x00020010
			set;
		}

		// Token: 0x17000276 RID: 630
		[HandlerParameter(false, "true", NotOptimized = true)]
		public bool TopBand
		{
			// Token: 0x060007D0 RID: 2000 RVA: 0x00021E19 File Offset: 0x00020019
			get;
			// Token: 0x060007D1 RID: 2001 RVA: 0x00021E21 File Offset: 0x00020021
			set;
		}
	}
}
