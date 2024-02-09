using System;
using System.CodeDom.Compiler;
using System.ComponentModel;
using System.Diagnostics;
using System.Globalization;
using System.Resources;
using System.Runtime.CompilerServices;

namespace vvTSLtools.Properties
{
	// Token: 0x02000086 RID: 134
	[GeneratedCode("System.Resources.Tools.StronglyTypedResourceBuilder", "4.0.0.0"), DebuggerNonUserCode, CompilerGenerated]
	internal class Resources
	{
		// Token: 0x0600049F RID: 1183 RVA: 0x00017C1B File Offset: 0x00015E1B
		internal Resources()
		{
		}

		// Token: 0x17000193 RID: 403
		[EditorBrowsable(EditorBrowsableState.Advanced)]
		internal static CultureInfo Culture
		{
			// Token: 0x060004A1 RID: 1185 RVA: 0x00017C63 File Offset: 0x00015E63
			get
			{
				return Resources.resourceCulture;
			}
			// Token: 0x060004A2 RID: 1186 RVA: 0x00017C6A File Offset: 0x00015E6A
			set
			{
				Resources.resourceCulture = value;
			}
		}

		// Token: 0x17000192 RID: 402
		[EditorBrowsable(EditorBrowsableState.Advanced)]
		internal static ResourceManager ResourceManager
		{
			// Token: 0x060004A0 RID: 1184 RVA: 0x00017C24 File Offset: 0x00015E24
			get
			{
				if (object.ReferenceEquals(Resources.resourceMan, null))
				{
					ResourceManager resourceManager = new ResourceManager("vvTSLtools.Properties.Resources", typeof(Resources).Assembly);
					Resources.resourceMan = resourceManager;
				}
				return Resources.resourceMan;
			}
		}

		// Token: 0x0400019B RID: 411
		private static CultureInfo resourceCulture;

		// Token: 0x0400019A RID: 410
		private static ResourceManager resourceMan;
	}
}
